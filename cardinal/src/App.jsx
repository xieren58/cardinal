import { useRef, useCallback, useEffect, useReducer, useState } from 'react';
import './App.css';
import { ContextMenu } from './components/ContextMenu';
import { ColumnHeader } from './components/ColumnHeader';
import { FileRow } from './components/FileRow';
import StatusBar from './components/StatusBar';
import { useColumnResize } from './hooks/useColumnResize';
import { useContextMenu } from './hooks/useContextMenu';
import { ROW_HEIGHT, OVERSCAN_ROW_COUNT, SEARCH_DEBOUNCE_MS } from './constants';
import { VirtualList } from './components/VirtualList';
import { StateDisplay } from './components/StateDisplay';
import FSEventsPanel from './components/FSEventsPanel';
import { usePreventRefresh } from './hooks/usePreventRefresh';
import { invoke } from '@tauri-apps/api/core';
import { listen, once } from '@tauri-apps/api/event';

const cancelTimer = (timerRef) => {
  if (timerRef.current) {
    clearTimeout(timerRef.current);
    timerRef.current = null;
  }
};

const INITIAL_EVENTS_FETCH = 200;
const initialState = {
  results: [],
  isInitialized: false,
  scannedFiles: 0,
  processedEvents: 0,
  currentQuery: '',
  showLoadingUI: false,
  initialFetchCompleted: false,
  durationMs: null,
  resultCount: 0,
  searchError: null,
};

function reducer(state, action) {
  switch (action.type) {
    case 'STATUS_UPDATE':
      return {
        ...state,
        scannedFiles: action.payload.scannedFiles,
        processedEvents: action.payload.processedEvents,
      };
    case 'INIT_COMPLETED':
      return { ...state, isInitialized: true };
    case 'SEARCH_REQUEST':
      return {
        ...state,
        searchError: null,
        showLoadingUI: action.payload.immediate ? true : state.showLoadingUI,
      };
    case 'SEARCH_LOADING_DELAY':
      return {
        ...state,
        showLoadingUI: true,
      };
    case 'SEARCH_SUCCESS':
      return {
        ...state,
        results: action.payload.results,
        currentQuery: action.payload.query,
        showLoadingUI: false,
        initialFetchCompleted: true,
        durationMs: action.payload.duration,
        resultCount: action.payload.count,
        searchError: null,
      };
    case 'SEARCH_FAILURE':
      return {
        ...state,
        showLoadingUI: false,
        searchError: action.payload.error,
        initialFetchCompleted: true,
        durationMs: action.payload.duration,
        resultCount: 0,
      };
    default:
      return state;
  }
}

function App() {
  usePreventRefresh();
  const [state, dispatch] = useReducer(reducer, initialState);
  const {
    results,
    isInitialized,
    scannedFiles,
    processedEvents,
    currentQuery,
    showLoadingUI,
    initialFetchCompleted,
    durationMs,
    resultCount,
    searchError,
  } = state;
  const [activeTab, setActiveTab] = useState('files');
  const eventsTotalRef = useRef(0);
  const updateEventsTotal = useCallback((value) => {
    const normalized = Number.isFinite(value) && value >= 0 ? value : 0;
    eventsTotalRef.current = normalized;
    setEventsTotal(normalized);
  }, []);
  const { colWidths, onResizeStart, autoFitColumns } = useColumnResize();
  const { menu, showContextMenu, showHeaderContextMenu, closeMenu, getMenuItems } =
    useContextMenu(autoFitColumns);
  const eventsCacheRef = useRef(new Map());
  const eventsLoadingRef = useRef(new Set());
  const eventsFetchVersionRef = useRef(0);
  const [eventsVersion, bumpEventsVersion] = useReducer((v) => v + 1, 0);
  const [eventsTotal, setEventsTotal] = useState(0);

  const headerRef = useRef(null);
  const virtualListRef = useRef(null);
  const isMountedRef = useRef(false);
  const debounceTimerRef = useRef(null);
  const loadingDelayTimerRef = useRef(null);
  const hasInitialSearchRunRef = useRef(false);
  const latestSearchRef = useRef({ query: '', useRegex: false, caseSensitive: false });
  const searchVersionRef = useRef(0);
  const [searchParams, updateSearchParams] = useReducer((prev, patch) => {
    const next = { ...prev, ...patch };
    latestSearchRef.current = next;
    return next;
  }, latestSearchRef.current);
  const { useRegex, caseSensitive } = searchParams;

  const resetEventsCache = useCallback(() => {
    eventsFetchVersionRef.current += 1;
    eventsCacheRef.current = new Map();
    eventsLoadingRef.current = new Set();
    bumpEventsVersion();
  }, []);

  const fetchEventsRange = useCallback(
    async (start, count) => {
      if (count <= 0) return;
      const requestVersion = eventsFetchVersionRef.current;
      try {
        const response = await invoke('get_recent_events_range', { start, count });
        if (!isMountedRef.current || requestVersion !== eventsFetchVersionRef.current) {
          return;
        }

        const total = Number(response?.total ?? eventsTotalRef.current);
        if (!Number.isNaN(total)) {
          updateEventsTotal(total);
        }

        const records = Array.isArray(response?.events) ? response.events : [];
        if (records.length > 0) {
          const next = new Map(eventsCacheRef.current);
          records.forEach((event, idx) => {
            next.set(start + idx, event);
          });
          eventsCacheRef.current = next;
          bumpEventsVersion();
        }
      } catch (error) {
        if (isMountedRef.current) {
          console.error('Failed to fetch recent events', error);
        }
      }
    },
    [updateEventsTotal],
  );

  const ensureEventsRange = useCallback(
    async (startIndex, stopIndex) => {
      const total = eventsTotalRef.current;
      if (total === 0) return;
      const start = Math.max(0, startIndex);
      const end = Math.min(stopIndex, total - 1);
      if (end < start) return;

      const missing = [];
      for (let i = start; i <= end; i += 1) {
        if (!eventsCacheRef.current.has(i) && !eventsLoadingRef.current.has(i)) {
          missing.push(i);
        }
      }

      if (missing.length === 0) return;

      const fetchStart = Math.min(...missing);
      const fetchEnd = Math.max(...missing) + 1;
      const count = fetchEnd - fetchStart;

      for (let i = fetchStart; i < fetchStart + count; i += 1) {
        eventsLoadingRef.current.add(i);
      }

      try {
        await fetchEventsRange(fetchStart, count);
      } finally {
        for (let i = fetchStart; i < fetchStart + count; i += 1) {
          eventsLoadingRef.current.delete(i);
        }
      }
    },
    [fetchEventsRange],
  );

  const getEventAt = useCallback(
    (index) => eventsCacheRef.current.get(index) ?? null,
    [eventsVersion],
  );

  useEffect(() => {
    isMountedRef.current = true;
    let unlistenStatus;
    let unlistenInit;

    const setupListeners = async () => {
      unlistenStatus = await listen('status_bar_update', (event) => {
        if (!isMountedRef.current) return;
        const { scanned_files, processed_events } = event.payload;
        dispatch({
          type: 'STATUS_UPDATE',
          payload: {
            scannedFiles: scanned_files,
            processedEvents: processed_events,
          },
        });
      });

      unlistenInit = await once('init_completed', () => {
        if (!isMountedRef.current) return;
        dispatch({ type: 'INIT_COMPLETED' });
      });
    };

    setupListeners();

    return () => {
      isMountedRef.current = false;
      if (typeof unlistenStatus === 'function') {
        unlistenStatus();
      }
      if (typeof unlistenInit === 'function') {
        unlistenInit();
      }
    };
  }, []);

  useEffect(() => {
    fetchEventsRange(0, INITIAL_EVENTS_FETCH);
  }, [fetchEventsRange]);

  useEffect(() => {
    let unlistenEvents;

    const setupEventsListener = async () => {
      try {
        unlistenEvents = await listen('fs_events_appended', (event) => {
          if (!isMountedRef.current) return;
          const totalCount = Number(event?.payload) || 0;
          updateEventsTotal(totalCount);
          resetEventsCache();
          if (totalCount > 0) {
            const initialCount = Math.min(totalCount, INITIAL_EVENTS_FETCH);
            fetchEventsRange(0, initialCount);
          }
        });
      } catch (error) {
        console.error('Failed to listen for file events', error);
      }
    };

    setupEventsListener();

    return () => {
      if (typeof unlistenEvents === 'function') {
        unlistenEvents();
      }
    };
  }, [fetchEventsRange, resetEventsCache, updateEventsTotal]);

  useEffect(() => {
    if (activeTab !== 'events') return;
    const total = eventsTotalRef.current;
    if (total === 0) return;
    const end = Math.min(total - 1, INITIAL_EVENTS_FETCH - 1);
    if (end >= 0) {
      ensureEventsRange(0, end);
    }
  }, [activeTab, ensureEventsRange, eventsTotal, eventsVersion]);

  const handleSearch = useCallback(
    async (overrides = {}) => {
      const nextSearch = { ...latestSearchRef.current, ...overrides };
      latestSearchRef.current = nextSearch;
      const requestVersion = searchVersionRef.current + 1;
      searchVersionRef.current = requestVersion;

      const { query, useRegex: nextUseRegex, caseSensitive: nextCaseSensitive } = nextSearch;
      const startTs = performance.now();
      const isInitial = !hasInitialSearchRunRef.current;
      const trimmedQuery = query.trim();

      dispatch({ type: 'SEARCH_REQUEST', payload: { immediate: isInitial } });

      if (!isInitial) {
        cancelTimer(loadingDelayTimerRef);
        loadingDelayTimerRef.current = setTimeout(() => {
          dispatch({ type: 'SEARCH_LOADING_DELAY' });
          loadingDelayTimerRef.current = null;
        }, 150);
      }

      try {
        const searchResults = await invoke('search', {
          query,
          options: {
            useRegex: nextUseRegex,
            caseInsensitive: !nextCaseSensitive,
          },
        });

        if (searchVersionRef.current !== requestVersion) {
          return;
        }

        cancelTimer(loadingDelayTimerRef);

        const endTs = performance.now();
        const duration = endTs - startTs;

        dispatch({
          type: 'SEARCH_SUCCESS',
          payload: {
            results: searchResults,
            query: trimmedQuery,
            duration,
            count: Array.isArray(searchResults) ? searchResults.length : 0,
          },
        });
      } catch (error) {
        console.error('Search failed:', error);

        if (searchVersionRef.current !== requestVersion) {
          return;
        }

        cancelTimer(loadingDelayTimerRef);

        const endTs = performance.now();
        const duration = endTs - startTs;

        dispatch({
          type: 'SEARCH_FAILURE',
          payload: {
            error: error || 'An unknown error occurred.',
            duration,
          },
        });
      } finally {
        hasInitialSearchRunRef.current = true;
      }
    },
    [dispatch],
  );

  const onQueryChange = useCallback(
    (e) => {
      const inputValue = e.target.value;
      updateSearchParams({ query: inputValue });
      cancelTimer(debounceTimerRef);
      debounceTimerRef.current = setTimeout(() => {
        handleSearch({ query: inputValue });
      }, SEARCH_DEBOUNCE_MS);
    },
    [handleSearch, updateSearchParams],
  );

  const onToggleRegex = useCallback(
    (event) => {
      const nextValue = event.target.checked;
      updateSearchParams({ useRegex: nextValue });
    },
    [updateSearchParams],
  );

  const onToggleCaseSensitive = useCallback(
    (event) => {
      const nextValue = event.target.checked;
      updateSearchParams({ caseSensitive: nextValue });
    },
    [updateSearchParams],
  );

  useEffect(
    () => () => {
      cancelTimer(debounceTimerRef);
      cancelTimer(loadingDelayTimerRef);
    },
    [],
  );

  useEffect(() => {
    if (!hasInitialSearchRunRef.current) {
      handleSearch({ query: '' });
      return;
    }
    handleSearch();
  }, [caseSensitive, handleSearch, useRegex]);

  // scroll position reset & initial data preload on `results` change
  useEffect(() => {
    const list = virtualListRef.current;
    if (!list) return;

    list.scrollToTop?.();

    if (!results.length || !list.ensureRangeLoaded) {
      return;
    }

    const preloadCount = Math.min(30, results.length);
    list.ensureRangeLoaded(0, preloadCount - 1);
  }, [results]);

  // 滚动同步处理 - 单向同步版本（Grid -> Header）
  const handleHorizontalSync = useCallback((scrollLeft) => {
    if (headerRef.current) headerRef.current.scrollLeft = scrollLeft;
  }, []);

  // 单元格渲染 - 使用 useCallback 避免 VirtualList 不必要的重渲染
  const renderRow = useCallback(
    (rowIndex, item, rowStyle) => (
      <FileRow
        key={rowIndex}
        item={item}
        rowIndex={rowIndex}
        style={{ ...rowStyle, width: 'var(--columns-total)' }}
        onContextMenu={showContextMenu}
        searchQuery={currentQuery}
        caseInsensitive={!caseSensitive}
      />
    ),
    [showContextMenu, currentQuery, caseSensitive],
  );

  const getDisplayState = () => {
    if (!initialFetchCompleted) return 'loading';
    if (showLoadingUI) return 'loading';
    if (searchError) return 'error';
    if (results.length === 0) return 'empty';
    return 'results';
  };

  const displayState = getDisplayState();

  return (
    <main className="container">
      <div className="search-container">
        <div className="search-bar">
          <input
            id="search-input"
            onChange={onQueryChange}
            placeholder="Search for files and folders..."
            spellCheck={false}
            autoCorrect="off"
            autoComplete="off"
            autoCapitalize="off"
          />
          <div className="search-options">
            <label className="search-option" title="Toggle case-sensitive matching">
              <input
                type="checkbox"
                checked={caseSensitive}
                onChange={onToggleCaseSensitive}
                aria-label="Toggle case-sensitive matching"
              />
              <span className="search-option__display" aria-hidden="true">
                Aa
              </span>
              <span className="sr-only">Toggle case-sensitive matching</span>
            </label>
            <label className="search-option" title="Enable regular expression search">
              <input
                type="checkbox"
                checked={useRegex}
                onChange={onToggleRegex}
                aria-label="Enable regular expression search"
              />
              <span className="search-option__display" aria-hidden="true">
                .*
              </span>
              <span className="sr-only">Enable regular expression search</span>
            </label>
          </div>
        </div>
      </div>
      <div
        className="results-container"
        style={{
          ['--w-filename']: `${colWidths.filename}px`,
          ['--w-path']: `${colWidths.path}px`,
          ['--w-size']: `${colWidths.size}px`,
          ['--w-modified']: `${colWidths.modified}px`,
          ['--w-created']: `${colWidths.created}px`,
        }}
      >
        {activeTab === 'events' ? (
          <div className="events-view">
            <FSEventsPanel
              totalCount={eventsTotal}
              getEvent={getEventAt}
              ensureRange={ensureEventsRange}
            />
          </div>
        ) : (
          <div className="scroll-area">
            <ColumnHeader
              ref={headerRef}
              onResizeStart={onResizeStart}
              onContextMenu={showHeaderContextMenu}
            />
            <div className="flex-fill">
              {displayState !== 'results' ? (
                <StateDisplay state={displayState} message={searchError} query={currentQuery} />
              ) : (
                <VirtualList
                  ref={virtualListRef}
                  results={results}
                  rowHeight={ROW_HEIGHT}
                  overscan={OVERSCAN_ROW_COUNT}
                  renderRow={renderRow}
                  onScrollSync={handleHorizontalSync}
                  className="virtual-list"
                />
              )}
            </div>
          </div>
        )}
      </div>
      {menu.visible && (
        <ContextMenu x={menu.x} y={menu.y} items={getMenuItems()} onClose={closeMenu} />
      )}
      <StatusBar
        scannedFiles={scannedFiles}
        processedEvents={processedEvents}
        isReady={isInitialized}
        searchDurationMs={durationMs}
        resultCount={resultCount}
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />
    </main>
  );
}

export default App;
