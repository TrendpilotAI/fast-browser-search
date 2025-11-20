import { useState, useEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { api, SearchResult } from '../lib/api';
import { Search, Globe, Chrome, Globe2, Compass, Star, Layers } from 'lucide-react';
import { format } from 'date-fns';
import { clsx } from 'clsx';

const IS_TAURI = typeof window !== 'undefined' && '__TAURI__' in window;

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const startTime = useRef<number>(0);

  // Search with performance tracking
  const { data, isLoading } = useQuery({
    queryKey: ['cmd-search', query],
    queryFn: async () => {
      if (!query) return null;
      startTime.current = Date.now();
      const result = await api.search(query, undefined, 50);
      const queryTime = Date.now() - startTime.current;
      return { ...result, ms: queryTime };
    },
    enabled: query.length > 0,
  });

  const results = data?.results || [];

  // Reset selection when results change
  useEffect(() => {
    setSelected(0);
  }, [results.length]);

  // Keyboard navigation
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Global ⌘K to toggle (only when not in input field or palette is already open)
      if (e.metaKey && e.key === 'k') {
        if (!open || document.activeElement !== inputRef.current) {
          e.preventDefault();
          setOpen(!open);
          // Focus input when opening
          if (!open) {
            setTimeout(() => inputRef.current?.focus(), 50);
          }
        }
      }

      if (!open) return;

      // Handle Escape - close palette and clear query on second press
      if (e.key === 'Escape') {
        e.preventDefault();
        if (query.length > 0) {
          setQuery('');
        } else {
          setOpen(false);
        }
      }

      // Navigation keys
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelected((i) => Math.min(i + 1, results.length - 1));
      }

      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelected((i) => Math.max(i - 1, 0));
      }

      // Open URL on Enter
      if (e.key === 'Enter' && results.length > 0) {
        e.preventDefault();
        const result = results[selected];
        if (result) {
          if (e.metaKey || e.ctrlKey) {
            // Open in new window
            window.open(result.url, '_blank', 'noopener,noreferrer');
          } else {
            // Open in same tab
            window.open(result.url, '_blank');
          }
          setOpen(false);
          setQuery('');
        }
      }

      // Copy URL on ⌘C (when not selecting text)
      if ((e.metaKey || e.ctrlKey) && e.key === 'c' && results.length > 0) {
        const selection = window.getSelection();
        if (!selection || selection.toString().length === 0) {
          e.preventDefault();
          const result = results[selected];
          if (result) {
            navigator.clipboard.writeText(result.url);
          }
        }
      }
    };

    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [open, query, selected, results]);

  // Tauri event listener for global hotkey
  useEffect(() => {
    if (!IS_TAURI) return;

    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      const { listen } = await import('@tauri-apps/api/event');
      unlisten = await listen('toggle-palette', () => {
        setOpen((prev) => !prev);
        if (!open) {
          setTimeout(() => inputRef.current?.focus(), 50);
        }
      });
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, [open]);

  // Close on backdrop click
  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      setOpen(false);
      setQuery('');
    }
  };

  // Get browser icon
  const getBrowserIcon = (browser: string) => {
    switch (browser.toLowerCase()) {
      case 'chrome':
        return <Chrome className="w-4 h-4 text-cmd-text-dim" />;
      case 'safari':
        return <Globe2 className="w-4 h-4 text-cmd-text-dim" />;
      case 'arc':
        return <Compass className="w-4 h-4 text-cmd-text-dim" />;
      case 'comet':
        return <Star className="w-4 h-4 text-cmd-text-dim" />;
      case 'thorium':
        return <Layers className="w-4 h-4 text-cmd-text-dim" />;
      default:
        return <Globe className="w-4 h-4 text-cmd-text-dim" />;
    }
  };

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/60 backdrop-blur-sm"
      onClick={handleBackdropClick}
    >
      <div className="w-[840px] max-w-[90vw] bg-cmd-surface rounded-2xl shadow-2xl animate-in overflow-hidden">
        {/* Input Header */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-cmd-border">
          <Search className="w-5 h-5 text-cmd-accent flex-shrink-0" />
          <input
            ref={inputRef}
            autoFocus
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search everywhere..."
            className="flex-1 bg-transparent text-[15px] text-cmd-text placeholder-cmd-text-muted outline-none"
          />
          {data && (
            <span className="text-xs text-cmd-text-muted flex-shrink-0">
              {data.query_time_ms?.toFixed(1) || data.ms?.toFixed(1) || '0'}ms • {data.total} items
            </span>
          )}
        </div>

        {/* Results */}
        <div className="max-h-[480px] overflow-auto">
          {isLoading && query.length > 0 && (
            <div className="px-4 py-8 text-center text-cmd-text-dim text-sm">
              Searching...
            </div>
          )}

          {!isLoading && query.length > 0 && results.length === 0 && (
            <div className="px-4 py-8 text-center">
              <p className="text-cmd-text-dim text-sm mb-2">No results found</p>
              <p className="text-cmd-text-muted text-xs">
                Try different keywords or check your filters
              </p>
            </div>
          )}

          {query.length === 0 && (
            <div className="px-4 py-8 text-center">
              <p className="text-cmd-text-dim text-sm mb-2">Start typing to search</p>
              <p className="text-cmd-text-muted text-xs">
                Search across your browser history
              </p>
            </div>
          )}

          {results.map((result, i) => (
            <div
              key={`${result.url}-${i}`}
              className={clsx(
                'flex items-center gap-3 px-4 py-3 cursor-pointer transition-colors',
                i === selected
                  ? 'bg-cmd-active border-l-2 border-cmd-accent'
                  : 'hover:bg-cmd-hover border-l-2 border-transparent'
              )}
              onClick={() => {
                window.open(result.url, '_blank');
                setOpen(false);
                setQuery('');
              }}
              onMouseEnter={() => setSelected(i)}
            >
              {getBrowserIcon(result.browser_source)}
              <div className="flex-1 min-w-0">
                <div className="text-[14px] font-medium text-cmd-text truncate">
                  {result.title || result.url}
                </div>
                <div className="text-[12px] text-cmd-text-dim truncate">
                  {result.domain} • {format(new Date(result.visit_time), 'MMM d, yyyy')} • {result.visit_count} visits
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Footer hint */}
        {results.length > 0 && (
          <div className="px-4 py-2 border-t border-cmd-border bg-cmd-bg">
            <div className="flex items-center justify-between text-[11px] text-cmd-text-muted">
              <div className="flex items-center gap-3">
                <span>↑↓ Navigate</span>
                <span>⏎ Open</span>
                <span>⌘⏎ New Window</span>
              </div>
              <span>ESC Close</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

