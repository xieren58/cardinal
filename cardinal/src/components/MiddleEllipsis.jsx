import React, { useEffect, useRef, useState } from 'react';

// Render text with a middle ellipsis that fits the available width of its grid cell.
export function MiddleEllipsis({ text, className }) {
  const containerRef = useRef(null);
  const [display, setDisplay] = useState(text || '');

  useEffect(() => {
    function compute() {
      const el = containerRef.current;
      if (!el) return;
      
      const str = text || '';
      if (!str) {
        setDisplay('');
        return;
      }

      // Simple character-based estimation
      const containerWidth = el.getBoundingClientRect().width;
      const charWidth = 8; // approximate character width in pixels
      const maxChars = Math.floor(containerWidth / charWidth) - 1; // -1 for ellipsis

      if (str.length <= maxChars) {
        setDisplay(str);
        return;
      }

      if (maxChars <= 2) {
        setDisplay('…');
        return;
      }

      // Keep equal parts from start and end
      const leftChars = Math.floor(maxChars / 2);
      const rightChars = maxChars - leftChars;
      
      const leftStr = str.slice(0, leftChars);
      const rightStr = str.slice(str.length - rightChars);
      setDisplay(`${leftStr}…${rightStr}`);
    }

    compute();
    const ro = new ResizeObserver(() => compute());
    const el = containerRef.current;
    if (el) ro.observe(el);
    return () => ro.disconnect();
  }, [text]);

  return (
    <span ref={containerRef} className={className} title={text}>
      {display}
    </span>
  );
}
