import React, { useCallback, useRef, useLayoutEffect, useState } from 'react';
import type { CSSProperties } from 'react';
import type { AppLifecycleStatus } from '../types/ipc';

export type StatusTabKey = 'files' | 'events';

type StatusBarProps = {
  scannedFiles: number;
  processedEvents: number;
  lifecycleState: AppLifecycleStatus;
  searchDurationMs?: number | null;
  resultCount?: number | null;
  activeTab?: StatusTabKey;
  onTabChange?: (tab: StatusTabKey) => void;
};

const TABS: Array<{ key: StatusTabKey; label: string }> = [
  { key: 'files', label: 'Files' },
  { key: 'events', label: 'Events' },
];

const LIFECYCLE_DISPLAY: Record<AppLifecycleStatus, { icon: string; label: string; tone: string }> = {
  Initializing: { icon: '○', label: 'Initializing', tone: 'initializing' },
  Ready: { icon: '●', label: 'Ready', tone: 'ready' },
  Closing: { icon: '●', label: 'Closing', tone: 'closing' },
};

const StatusBar = ({
  scannedFiles,
  processedEvents,
  lifecycleState,
  searchDurationMs,
  resultCount,
  activeTab = 'files',
  onTabChange,
}: StatusBarProps): React.JSX.Element => {
  const tabsRef = useRef<HTMLDivElement | null>(null);
  const filesTabRef = useRef<HTMLButtonElement | null>(null);
  const eventsTabRef = useRef<HTMLButtonElement | null>(null);
  const [sliderStyle, setSliderStyle] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    // Keep the active-tab underline aligned even when labels resize or counters update.
    const updateSliderPosition = () => {
      const activeTabRef = activeTab === 'files' ? filesTabRef : eventsTabRef;
      if (activeTabRef.current && tabsRef.current) {
        const tabRect = activeTabRef.current.getBoundingClientRect();
        const containerRect = tabsRef.current.getBoundingClientRect();

        setSliderStyle({
          left: `${tabRect.left - containerRect.left}px`,
          width: `${tabRect.width}px`,
        });
      }
    };

    updateSliderPosition();
    // Re-align the slider when the viewport changes width.
    window.addEventListener('resize', updateSliderPosition);
    return () => window.removeEventListener('resize', updateSliderPosition);
  }, [activeTab, scannedFiles, processedEvents]);

  const handleSelect = useCallback(
    (tabKey: StatusTabKey) => {
      if (tabKey === activeTab) return;
      onTabChange?.(tabKey);
    },
    [activeTab, onTabChange],
  );

  const resultsText =
    typeof resultCount === 'number'
      ? `${resultCount.toLocaleString()} result${resultCount === 1 ? '' : 's'}`
      : '—';
  const durationText =
    typeof searchDurationMs === 'number' ? `${Math.round(searchDurationMs)}ms` : null;
  const searchDisplay = durationText ? `${resultsText} • ${durationText}` : resultsText;
  const lifecycle = LIFECYCLE_DISPLAY[lifecycleState] ?? LIFECYCLE_DISPLAY.Initializing;

  return (
    <div className="status-bar">
      <div className="status-left">
        <div className="status-section">
          <span
            className={`readiness-indicator ${lifecycle.tone}`}
            aria-label={`Application status: ${lifecycle.label}`}
            title={`Application status: ${lifecycle.label}`}
          >
            {lifecycle.icon}
          </span>
          <span className="status-text">{lifecycle.label}</span>
        </div>
        <div ref={tabsRef} className="status-tabs" role="tablist" aria-label="Search status view">
          <div className="status-tabs-slider" style={sliderStyle} />
          {TABS.map(({ key, label }) => {
            const isActive = activeTab === key;
            const value =
              key === 'files' ? scannedFiles.toLocaleString() : processedEvents.toLocaleString();
            const ref = key === 'files' ? filesTabRef : eventsTabRef;
            return (
              <button
                key={key}
                ref={ref}
                type="button"
                role="tab"
                aria-selected={isActive}
                className={`status-tab ${isActive ? 'is-active' : ''}`}
                data-tone={key}
                onClick={() => handleSelect(key)}
              >
                <span className="status-tab__label">{label}</span>
                <span className="status-tab__value">{value}</span>
              </button>
            );
          })}
        </div>
      </div>

      <div className="status-right">
        <div className="status-section">
          <span className="status-label">Search:</span>
          <span className="status-value" title="Results • Duration">
            {searchDisplay}
          </span>
        </div>
      </div>
    </div>
  );
};

export default StatusBar;
