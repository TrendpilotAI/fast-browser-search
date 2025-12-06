import { useState, useEffect, useRef } from 'react';
import { 
  Search, Mail, Globe, ArrowRight, Clock, 
  Cpu, Sparkles, RefreshCw, Zap, X
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { useMainWindowSearch } from '../hooks/useMainWindowSearch';
import { SearchResult } from '../lib/api';
import { connectGoogle, reIndex, getGmailStatus } from '../lib/tauri';
import { highlightSearchTerms } from '../utils/highlight';
import { generateRelevanceExplanation } from '../utils/relevance';
import clsx from 'clsx';

// --- Icons & Visuals ---

const SourceIcon = ({ source }: { source: string }) => {
  switch (source) {
    case 'gmail': return <Mail size={16} className="text-red-400" />;
    case 'chrome': return <Globe size={16} className="text-blue-400" />;
    case 'arc': return <Sparkles size={16} className="text-pink-400" />;
    case 'safari': return <Globe size={16} className="text-blue-300" />;
    case 'history': return <Clock size={16} className="text-text-muted" />;
    default: return <Globe size={16} className="text-emerald-400" />;
  }
};

const BrowserIcon = ({ browser }: { browser: string }) => {
  switch (browser.toLowerCase()) {
    case 'chrome': return <Globe size={14} className="text-blue-400" />;
    case 'safari': return <Globe size={14} className="text-blue-300" />;
    case 'arc': return <Sparkles size={14} className="text-pink-400" />;
    case 'comet': return <Sparkles size={14} className="text-purple-400" />;
    case 'genspark': return <Globe size={14} className="text-green-400" />;
    case 'thorium': return <Globe size={14} className="text-orange-400" />;
    case 'gmail': return <Mail size={14} className="text-red-400" />;
    default: return <Globe size={14} />;
  }
};

// --- Result Row Component ---

const ResultRow = ({ 
  item, 
  query,
  selected, 
  onClick, 
  onMouseEnter 
}: { 
  item: SearchResult;
  query: string;
  selected: boolean; 
  onClick: () => void;
  onMouseEnter: () => void;
}) => {
  const relevanceExplanation = generateRelevanceExplanation(item, query);
  
  return (
    <motion.div 
      layout
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{ duration: 0.15 }}
      onClick={onClick}
      onMouseEnter={onMouseEnter}
      className={clsx(
        "relative px-4 py-3 rounded-lg cursor-pointer transition-all",
        selected ? "bg-surface-active border border-border-focus" : "bg-surface border border-transparent hover:border-border-subtle"
      )}
    >
      <div className="flex items-start gap-4">
        {/* Icon */}
        <div className={clsx(
          "w-10 h-10 rounded-xl flex items-center justify-center border transition-all shrink-0",
          selected ? "bg-bg-elevated border-border-focus" : "bg-surface-hover border-border-subtle"
        )}>
          <SourceIcon source={item.source} />
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          {/* Title with highlighting */}
          <div className="flex items-center gap-2 mb-1">
            <h3 className={clsx(
              "text-[15px] font-medium truncate",
              selected ? "text-text-primary" : "text-text-secondary"
            )}>
              {highlightSearchTerms(item.title || item.url, query)}
            </h3>
            
            {/* Site Category Badge */}
            {item.site_category && (
              <span className="px-2 py-0.5 rounded-md bg-bg-primary border border-border-subtle text-[10px] text-text-muted uppercase tracking-wider font-medium">
                {item.site_category}
              </span>
            )}
          </div>

          {/* Summary with highlighting */}
          {item.summary && (
            <p className="text-[13px] text-text-secondary mb-2 line-clamp-2">
              {highlightSearchTerms(item.summary, query)}
            </p>
          )}

          {/* Relevance Explanation */}
          <p className="text-[12px] text-text-muted mb-2">
            {relevanceExplanation}
          </p>

          {/* Metadata Row */}
          <div className="flex items-center gap-3 text-[11px] text-text-muted">
            <span className="flex items-center gap-1">
              <Clock size={12} />
              {item.last_visit ? (() => {
                const date = new Date(item.last_visit);
                const diff = Date.now() - date.getTime();
                if (diff < 24 * 60 * 60 * 1000) {
                  return 'Today';
                } else if (diff < 48 * 60 * 60 * 1000) {
                  return 'Yesterday';
                } else {
                  return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
                }
              })() : 'Recently'}
            </span>
            {item.visit_count !== undefined && item.visit_count > 0 && (
              <>
                <span>•</span>
                <span>{item.visit_count} {item.visit_count === 1 ? 'visit' : 'visits'}</span>
              </>
            )}
            {item.score !== undefined && (
              <>
                <span>•</span>
                <span className="flex items-center gap-1">
                  <span className="w-12 h-1 bg-bg-primary rounded-full overflow-hidden">
                    <div 
                      className="h-full bg-accent-primary rounded-full" 
                      style={{ width: `${Math.min(item.score * 100, 100)}%` }} 
                    />
                  </span>
                  {Math.round(item.score * 100)}% match
                </span>
              </>
            )}
          </div>

          {/* Key Topics */}
          {item.key_topics && item.key_topics.length > 0 && (
            <div className="flex flex-wrap gap-1.5 mt-2">
              {item.key_topics.slice(0, 3).map((topic, idx) => (
                <span 
                  key={idx}
                  className="px-1.5 py-0.5 rounded-md bg-bg-primary border border-border-subtle text-[10px] text-text-muted"
                >
                  {topic}
                </span>
              ))}
            </div>
          )}
        </div>

        {/* Action Arrow */}
        {selected && (
          <motion.div
            initial={{ opacity: 0, x: -5 }}
            animate={{ opacity: 1, x: 0 }}
            className="shrink-0"
          >
            <div className="w-8 h-8 rounded-lg flex items-center justify-center bg-accent-primary text-white">
              <ArrowRight size={14} />
            </div>
          </motion.div>
        )}
      </div>
    </motion.div>
  );
};

// --- Browser Filter Chip ---

const BrowserFilterChip = ({ 
  browser, 
  active, 
  onClick 
}: { 
  browser: string; 
  active: boolean; 
  onClick: () => void;
}) => {
  return (
    <button
      onClick={onClick}
      className={clsx(
        "px-4 py-2 rounded-lg border transition-all flex items-center gap-2 text-sm font-medium",
        active 
          ? "bg-accent-primary/20 border-accent-primary text-accent-primary" 
          : "bg-surface border-border-subtle text-text-secondary hover:bg-surface-hover"
      )}
    >
      <BrowserIcon browser={browser} />
      <span className="capitalize">{browser}</span>
    </button>
  );
};

// --- Main Component ---

export const MainWindow = () => {
  const { 
    query, setQuery, results, isLoading, 
    selectedIndex, setSelectedIndex, handleNavigation, addToHistory,
    browserFilters, toggleBrowserFilter
  } = useMainWindowSearch();
  
  const [gmailConnected, setGmailConnected] = useState(false);
  const [gmailLoading, setGmailLoading] = useState(false);
  const [indexLoading, setIndexLoading] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  
  const inputRef = useRef<HTMLInputElement>(null);
  const resultsRef = useRef<HTMLDivElement>(null);

  // Check Gmail status on mount
  useEffect(() => {
    const checkGmailStatus = async () => {
      try {
        const status = await getGmailStatus();
        setGmailConnected(status);
      } catch (error) {
        console.error('Failed to check Gmail status:', error);
      }
    };
    checkGmailStatus();
  }, []);

  // Handle Connect Gmail
  const handleConnectGmail = async () => {
    setGmailLoading(true);
    setMessage(null);
    try {
      await connectGoogle();
      setMessage({ type: 'success', text: 'Gmail connection initiated. Please complete OAuth in your browser.' });
      // Check status after a delay
      setTimeout(async () => {
        const status = await getGmailStatus();
        setGmailConnected(status);
      }, 2000);
    } catch (error: any) {
      setMessage({ type: 'error', text: error.message || 'Failed to connect Gmail' });
    } finally {
      setGmailLoading(false);
    }
  };

  // Handle Re-index
  const handleReIndex = async () => {
    setIndexLoading(true);
    setMessage(null);
    try {
      await reIndex();
      setMessage({ type: 'success', text: 'Re-indexing started in background...' });
    } catch (error: any) {
      setMessage({ type: 'error', text: error.message || 'Failed to start re-indexing' });
    } finally {
      setIndexLoading(false);
    }
  };

  // Handle result click
  const handleResultClick = (item: SearchResult) => {
    if (item.url) {
      addToHistory(query || item.title);
      window.open(item.url, '_blank');
    }
  };

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Enter' && results[selectedIndex]) {
        handleResultClick(results[selectedIndex]);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedIndex, results]);

  // Auto-scroll to selected result
  useEffect(() => {
    if (resultsRef.current && selectedIndex >= 0) {
      const selectedElement = resultsRef.current.children[selectedIndex] as HTMLElement;
      if (selectedElement) {
        selectedElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }
    }
  }, [selectedIndex]);

  return (
    <div className="min-h-screen w-full bg-bg-primary text-text-primary font-sans antialiased">
      {/* Header */}
      <header className="border-b border-border-subtle bg-bg-elevated">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between mb-4">
            {/* Title Section */}
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2">
                <Zap size={24} className="text-accent-primary" />
                <h1 className="text-2xl font-bold text-text-primary">Ultra Fast Search</h1>
              </div>
              <p className="text-sm text-text-muted">Press ⌘K for quick search</p>
            </div>

            {/* Action Buttons */}
            <div className="flex items-center gap-3">
              <button
                onClick={handleConnectGmail}
                disabled={gmailLoading || gmailConnected}
                className={clsx(
                  "px-4 py-2 rounded-lg border transition-all flex items-center gap-2 text-sm font-medium",
                  gmailConnected
                    ? "bg-green-500/20 border-green-500 text-green-400 cursor-not-allowed"
                    : gmailLoading
                    ? "bg-surface border-border-subtle text-text-muted cursor-wait"
                    : "bg-red-500/20 border-red-500 text-red-400 hover:bg-red-500/30"
                )}
              >
                <Mail size={16} />
                {gmailLoading ? 'Connecting...' : gmailConnected ? 'Gmail Connected' : 'Connect Gmail'}
              </button>
              
              <button
                onClick={handleReIndex}
                disabled={indexLoading}
                className={clsx(
                  "px-4 py-2 rounded-lg border transition-all flex items-center gap-2 text-sm font-medium",
                  indexLoading
                    ? "bg-surface border-border-subtle text-text-muted cursor-wait"
                    : "bg-accent-primary/20 border-accent-primary text-accent-primary hover:bg-accent-primary/30"
                )}
              >
                <RefreshCw size={16} className={indexLoading ? "animate-spin" : ""} />
                Re-index
              </button>
            </div>
          </div>

          {/* Message Banner */}
          <AnimatePresence>
            {message && (
              <motion.div
                initial={{ opacity: 0, y: -10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                className={clsx(
                  "px-4 py-2 rounded-lg mb-4 flex items-center justify-between",
                  message.type === 'success' 
                    ? "bg-green-500/20 border border-green-500/50 text-green-400"
                    : "bg-red-500/20 border border-red-500/50 text-red-400"
                )}
              >
                <span className="text-sm">{message.text}</span>
                <button
                  onClick={() => setMessage(null)}
                  className="hover:opacity-70"
                >
                  <X size={16} />
                </button>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Search Input */}
          <div className="relative mb-4">
            <div className="relative">
              <Search 
                size={20} 
                className={clsx(
                  "absolute left-4 top-1/2 -translate-y-1/2",
                  isLoading ? "text-accent-primary animate-pulse" : "text-text-muted"
                )}
              />
              <input
                ref={inputRef}
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={handleNavigation}
                placeholder="Search here or press ⌘K for quick search..."
                className={clsx(
                  "w-full pl-12 pr-4 py-4 rounded-xl border bg-surface text-text-primary placeholder-text-muted/50",
                  "focus:outline-none focus:ring-2 focus:ring-accent-primary/50 focus:border-accent-primary",
                  "text-lg"
                )}
              />
              {isLoading && (
                <div className="absolute right-4 top-1/2 -translate-y-1/2">
                  <Cpu size={20} className="text-accent-primary animate-spin" />
                </div>
              )}
            </div>
          </div>

          {/* Browser Filter Chips */}
          <div className="flex items-center gap-2 flex-wrap">
            <BrowserFilterChip 
              browser="Chrome" 
              active={browserFilters.chrome} 
              onClick={() => toggleBrowserFilter('chrome')} 
            />
            <BrowserFilterChip 
              browser="Safari" 
              active={browserFilters.safari} 
              onClick={() => toggleBrowserFilter('safari')} 
            />
            <BrowserFilterChip 
              browser="Arc" 
              active={browserFilters.arc} 
              onClick={() => toggleBrowserFilter('arc')} 
            />
            <BrowserFilterChip 
              browser="Comet" 
              active={browserFilters.comet} 
              onClick={() => toggleBrowserFilter('comet')} 
            />
            <BrowserFilterChip 
              browser="Genspark" 
              active={browserFilters.genspark} 
              onClick={() => toggleBrowserFilter('genspark')} 
            />
            <BrowserFilterChip 
              browser="Thorium" 
              active={browserFilters.thorium} 
              onClick={() => toggleBrowserFilter('thorium')} 
            />
            <BrowserFilterChip 
              browser="Gmail" 
              active={browserFilters.gmail} 
              onClick={() => toggleBrowserFilter('gmail')} 
            />
          </div>
        </div>
      </header>

      {/* Results Area */}
      <main className="max-w-7xl mx-auto px-6 py-6">
        {results.length === 0 && !isLoading && !query && (
          <div className="flex flex-col items-center justify-center py-20 text-center">
            <div className="w-16 h-16 rounded-2xl bg-surface border border-border-subtle flex items-center justify-center mb-6">
              <Search size={32} className="text-accent-primary" />
            </div>
            <h3 className="text-lg font-medium text-text-primary mb-2">Start searching</h3>
            <p className="text-sm text-text-muted max-w-md">
              Type anything to search across Chrome, Safari, Arc, and Gmail instantly.
            </p>
          </div>
        )}

        {results.length === 0 && !isLoading && query && (
          <div className="flex flex-col items-center justify-center py-20 text-center">
            <Search size={48} className="text-text-muted mb-4" />
            <p className="text-text-muted">No matches found for "{query}"</p>
          </div>
        )}

        {results.length > 0 && (
          <div ref={resultsRef} className="space-y-2">
            <AnimatePresence>
              {results.map((item, index) => (
                <ResultRow
                  key={item.id || index}
                  item={item}
                  query={query}
                  selected={index === selectedIndex}
                  onClick={() => handleResultClick(item)}
                  onMouseEnter={() => setSelectedIndex(index)}
                />
              ))}
            </AnimatePresence>
          </div>
        )}
      </main>
    </div>
  );
};

