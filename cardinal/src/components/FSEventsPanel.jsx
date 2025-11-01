import React, { useMemo, useCallback, useRef, memo } from 'react';
import { VirtualList } from './VirtualList';
import { ROW_HEIGHT } from '../constants';

const COLUMNS = [
  { key: 'name', label: 'Name' },
  { key: 'path', label: 'Path' },
  { key: 'time', label: 'Time' },
];

const splitPath = (path) => {
  if (!path) {
    return { name: '—', directory: '' };
  }
  const normalized = path.replace(/\\/g, '/');
  if (normalized === '/') {
    return { name: '/', directory: '/' };
  }
  const slashIndex = normalized.lastIndexOf('/');
  if (slashIndex === -1) {
    return { name: normalized, directory: '' };
  }
  const directory = normalized.slice(0, slashIndex) || '/';
  const name = normalized.slice(slashIndex + 1) || normalized;
  return { name, directory };
};

const toDate = (timestamp) => {
  if (typeof timestamp !== 'number' || Number.isNaN(timestamp) || !Number.isFinite(timestamp)) {
    return null;
  }
  return new Date(timestamp * 1000);
};

const buildRelativeTimeFormatter = () =>
  new Intl.RelativeTimeFormat(undefined, { numeric: 'auto', style: 'long' });

const formatRelativeTime = (timestamp, formatter) => {
  const date = toDate(timestamp);
  if (!date) return '';
  const now = Date.now();
  const diffMs = date.getTime() - now;
  const diffSeconds = Math.round(diffMs / 1000);
  const absSeconds = Math.abs(diffSeconds);

  if (absSeconds < 60) {
    return formatter.format(diffSeconds, 'second');
  }
  const diffMinutes = Math.round(diffSeconds / 60);
  if (Math.abs(diffMinutes) < 120) {
    return formatter.format(diffMinutes, 'minute');
  }
  const diffHours = Math.round(diffMinutes / 60);
  if (Math.abs(diffHours) < 48) {
    return formatter.format(diffHours, 'hour');
  }
  const diffDays = Math.round(diffHours / 24);
  return formatter.format(diffDays, 'day');
};

// EventRow component for rendering individual rows
const EventRow = memo(function EventRow({ item: event, rowIndex, style }) {
  const pathSource = event?.path ?? '';
  const { name, directory } = splitPath(pathSource);
  const timestamp = event?.timestamp;
  
  const dateFormatter = useMemo(
    () =>
      new Intl.DateTimeFormat(undefined, {
        year: 'numeric',
        month: 'short',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      }),
    [],
  );
  
  const relativeFormatter = useMemo(() => new Intl.RelativeTimeFormat(undefined, { numeric: 'auto', style: 'long' }), []);
  
  const date = toDate(timestamp);
  const formattedDate = date ? dateFormatter.format(date) : '—';
  const relative = date ? formatRelativeTime(timestamp, relativeFormatter) : '';
  const isLoading = !event;

  return (
    <div
      style={style}
      className={`row columns-events ${rowIndex % 2 === 0 ? 'row-even' : 'row-odd'}`}
      title={pathSource}
    >
      <div className="event-name-column">
        {isLoading ? (
          <span className="events-loading">Loading…</span>
        ) : (
          <span className="event-name-text">{name || '—'}</span>
        )}
      </div>
      <span className="event-path-text" title={directory}>
        {isLoading ? '' : directory || (pathSource ? '/' : '—')}
      </span>
      <div className="event-time-column">
        {isLoading ? (
          ''
        ) : (
          <>
            <span className="event-time-primary">{formattedDate}</span>
            {relative && <span className="event-time-secondary">{relative}</span>}
          </>
        )}
      </div>
    </div>
  );
});

const FSEventsPanel = ({ totalCount, getEvent, ensureRange }) => {
  const dateFormatter = useMemo(
    () =>
      new Intl.DateTimeFormat(undefined, {
        year: 'numeric',
        month: 'short',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      }),
    [],
  );
  const relativeFormatter = useMemo(buildRelativeTimeFormatter, []);

  const handleSectionRendered = useCallback(
    ({ rowStartIndex, rowStopIndex }) => {
      if (rowStartIndex <= rowStopIndex) {
        ensureRange(rowStartIndex, rowStopIndex);
      }
    },
    [ensureRange],
  );

  const cellRenderer = useCallback(
    ({ columnIndex, rowIndex, key, style }) => {
      const event = getEvent(rowIndex);
      const pathSource = event?.path ?? '';
      const { name, directory } = splitPath(pathSource);
      const timestamp = event?.timestamp;
      const date = toDate(timestamp);
      const formattedDate = date ? dateFormatter.format(date) : '—';
      const relative = date ? formatRelativeTime(timestamp, relativeFormatter) : '';
      const isLoading = !event;
      const columnKey = COLUMNS[columnIndex].key;
      const parityClass = rowIndex % 2 === 0 ? 'is-even' : 'is-odd';

      let content;
      if (isLoading) {
        content = <span className="events-cell__loading">Loading…</span>;
      } else if (columnKey === 'name') {
        content = (
          <div className="events-cell__name" title={pathSource}>
            <span className="events-cell__primary">{name}</span>
          </div>
        );
      } else if (columnKey === 'path') {
        content = (
          <div className="events-cell__path" title={pathSource}>
            <span className="events-cell__secondary">{directory || (pathSource ? '/' : '—')}</span>
          </div>
        );
      } else {
        content = (
          <div className="events-cell__time" title={formattedDate}>
            <span className="events-cell__primary">{formattedDate}</span>
            {relative ? <span className="events-cell__secondary">{relative}</span> : null}
          </div>
        );
      }

      return (
        <div
          key={key}
          style={style}
          className={`events-cell events-cell--${columnKey} ${parityClass}${
            isLoading ? ' is-loading' : ''
          }`}
        >
          {content}
        </div>
      );
    },
    [dateFormatter, getEvent, relativeFormatter],
  );

  return (
    <div className="events-panel">
      <AutoSizer>
        {({ width, height }) => {
          const columnWidths = clampColumnWidths(width);
          const bodyHeight = Math.max(0, height - HEADER_HEIGHT);

          return (
            <div className="events-surface" style={{ width, height }}>
              <div className="events-header" style={{ height: HEADER_HEIGHT }}>
                {COLUMNS.map((column, index) => (
                  <div
                    key={column.key}
                    className="events-header__cell"
                    style={{ width: columnWidths[index] }}
                  >
                    {column.label}
                  </div>
                ))}
              </div>
              <div className="events-grid" style={{ height: bodyHeight }}>
                <Grid
                  width={width}
                  height={bodyHeight}
                  rowCount={totalCount}
                  columnCount={COLUMNS.length}
                  columnWidth={({ index }) => columnWidths[index]}
                  rowHeight={ROW_HEIGHT}
                  cellRenderer={cellRenderer}
                  overscanRowCount={6}
                  onSectionRendered={handleSectionRendered}
                />
                {totalCount === 0 && (
                  <div className="events-empty" role="status">
                    <p>No recent file events yet.</p>
                    <p className="events-empty__hint">Keep working and check back for updates.</p>
                  </div>
                )}
              </div>
            </div>
          );
        }}
      </AutoSizer>
    </div>
  );
};

export default FSEventsPanel;
