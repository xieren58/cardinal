import React from 'react';

const StatusBar = ({ scannedFiles, processedEvents, isReady, searchDurationMs, resultCount }) => {
  const resultsText =
    typeof resultCount === 'number'
      ? `${resultCount.toLocaleString()} result${resultCount === 1 ? '' : 's'}`
      : '—';
  const durationText = searchDurationMs != null ? `${Math.round(searchDurationMs)}ms` : null;
  const searchDisplay = durationText ? `${resultsText} • ${durationText}` : resultsText;

  return (
    <div className="status-bar">
      <div className="status-left">
        <div className="status-section">
          <span className={`readiness-indicator ${isReady ? 'ready' : 'not-ready'}`}>
            {isReady ? '●' : '○'}
          </span>
          <span className="status-text">{isReady ? 'Ready' : 'Initializing'}</span>
        </div>
        <div className="status-section">
          <span className="status-label">Files:</span>
          <span className="status-value">{scannedFiles.toLocaleString()}</span>
        </div>
        <div className="status-section">
          <span className="status-label">Events:</span>
          <span className="status-value">{processedEvents.toLocaleString()}</span>
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
