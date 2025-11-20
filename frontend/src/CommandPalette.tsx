/**
 * CommandPalette Component
 * Master-class UI with Framer Motion physics, ghost text, and micro-interactions
 */

import { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence, LayoutGroup, useMotionValue, useSpring, useTransform } from 'framer-motion';
import { Search, Clock, Globe, Chrome, Compass, Sparkles, TrendingUp, X, ArrowRight, Check, AlertCircle } from 'lucide-react';
import { format } from 'date-fns';
import { useCommandEngine } from './useCommandEngine';
import { easings, durations } from './tokens';
import { recentSearches } from './api';

// Browser icon mapping
const getBrowserIcon = (browser: string) => {
  switch (browser.toLowerCase()) {
    case 'chrome': return Chrome;
    case 'safari': return Globe;
    case 'arc': return Compass;
    default: return Globe;
  }
};

// Ghost Input Component (inline autocomplete)
function GhostInput({ 
  input, 
  suggestions 
}: { 
  input: string; 
  suggestions: string[] 
}) {
  const ghostText = suggestions.find(s => 
    s.toLowerCase().startsWith(input.toLowerCase()) && s !== input
  );

  if (!ghostText || !input) return null;

  return (
    <span 
      className="absolute left-12 top-1/2 -translate-y-1/2 text-gray-400 dark:text-gray-500 pointer-events-none select-none"
      style={{ 
        fontFamily: 'inherit',
        fontSize: 'inherit',
        lineHeight: 'inherit',
      }}
    >
      {ghostText.slice(input.length)}
    </span>
  );
}

// Result Row Component
function ResultRow({ 
  result, 
  index, 
  isSelected, 
  onSelect, 
  onNavigate 
}: { 
  result: any; 
  index: number; 
  isSelected: boolean; 
  onSelect: () => void; 
  onNavigate: () => void;
}) {
  const BrowserIcon = getBrowserIcon(result.browser_source);
  const [isHovered, setIsHovered] = useState(false);

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{
        duration: durations.fast / 1000,
        ease: easings.snappy,
        delay: index * 0.02,
      }}
      onHoverStart={() => setIsHovered(true)}
      onHoverEnd={() => setIsHovered(false)}
      onClick={onNavigate}
      onMouseEnter={onSelect}
      className="relative cursor-pointer"
    >
      {/* Selection Highlight with layoutId for fluid morphing */}
      {isSelected && (
        <motion.div
          layoutId="highlight"
          className="absolute inset-0 bg-blue-50 dark:bg-blue-900/20 rounded-lg"
          transition={{
            type: 'spring',
            stiffness: 300,
            damping: 30,
          }}
          style={{
            boxShadow: isSelected ? '0 0 0 2px rgba(59, 130, 246, 0.2)' : 'none',
          }}
        />
      )}

      <div className="relative flex items-center gap-3 px-4 py-3">
        {/* Icon with micro-interaction */}
        <motion.div
          animate={{
            scale: isHovered ? 1.1 : 1,
            rotate: isHovered ? [0, -5, 5, -5, 0] : 0,
          }}
          transition={{
            duration: 0.3,
            ease: easings.spring,
          }}
        >
          <BrowserIcon className="w-5 h-5 text-gray-600 dark:text-gray-400" />
        </motion.div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
              {result.title || new URL(result.url).hostname}
            </h3>
            {isSelected && (
              <motion.div
                initial={{ scale: 0, rotate: -180 }}
                animate={{ scale: 1, rotate: 0 }}
                transition={{ type: 'spring', stiffness: 400, damping: 20 }}
              >
                <Check className="w-4 h-4 text-blue-500" />
              </motion.div>
            )}
          </div>
          <p className="text-xs text-gray-600 dark:text-gray-400 truncate">
            {result.url}
          </p>
          <div className="flex items-center gap-3 mt-1 text-xs text-gray-500 dark:text-gray-500">
            <span className="flex items-center gap-1">
              <Clock className="w-3 h-3" />
              {format(new Date(result.visit_time), 'MMM d')}
            </span>
            <span>{result.visit_count} visits</span>
            {result.relevance_score > 0 && (
              <span className="text-blue-500">
                {Math.round(result.relevance_score * 100)}% match
              </span>
            )}
          </div>
        </div>

        {/* Arrow indicator */}
        <motion.div
          animate={{
            x: isHovered ? 4 : 0,
            opacity: isHovered ? 1 : 0.5,
          }}
          transition={{ duration: durations.fast / 1000, ease: easings.snappy }}
        >
          <ArrowRight className="w-4 h-4 text-gray-400 dark:text-gray-500" />
        </motion.div>
      </div>
    </motion.div>
  );
}

// Preview Pane Component
function PreviewPane({ 
  result, 
  isVisible 
}: { 
  result: any | null; 
  isVisible: boolean;
}) {
  if (!result) return null;

  const BrowserIcon = getBrowserIcon(result.browser_source);

  return (
    <AnimatePresence>
      {isVisible && result && (
        <motion.div
          initial={{ opacity: 0, x: 20, scale: 0.95 }}
          animate={{ opacity: 1, x: 0, scale: 1 }}
          exit={{ opacity: 0, x: 20, scale: 0.95 }}
          transition={{
            type: 'spring',
            stiffness: 300,
            damping: 30,
          }}
          className="absolute right-0 top-0 bottom-0 w-96 bg-white dark:bg-gray-800 border-l border-gray-200 dark:border-gray-700 p-6 overflow-y-auto"
        >
          <div className="space-y-4">
            <div>
              <div className="flex items-center gap-2 mb-2">
                <BrowserIcon className="w-5 h-5 text-gray-600 dark:text-gray-400" />
                <span className="text-xs text-gray-600 dark:text-gray-400">{result.browser_source}</span>
              </div>
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
                {result.title || new URL(result.url).hostname}
              </h2>
              <a
                href={result.url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-blue-500 hover:underline break-all"
              >
                {result.url}
              </a>
            </div>

            <div className="space-y-2 text-sm">
              <div className="flex items-center justify-between">
                <span className="text-gray-600 dark:text-gray-400">Visits</span>
                <span className="text-gray-900 dark:text-gray-100 font-medium">{result.visit_count}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-gray-600 dark:text-gray-400">Last visited</span>
                <span className="text-gray-900 dark:text-gray-100">
                  {format(new Date(result.visit_time), 'MMM d, yyyy')}
                </span>
              </div>
              {result.relevance_score > 0 && (
                <div className="flex items-center justify-between">
                  <span className="text-gray-600 dark:text-gray-400">Relevance</span>
                  <span className="text-gray-900 dark:text-gray-100 font-medium">
                    {Math.round(result.relevance_score * 100)}%
                  </span>
                </div>
              )}
            </div>

            {result.related_urls && result.related_urls.length > 0 && (
              <div>
                <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2">Related</h3>
                <div className="space-y-1">
                  {result.related_urls.slice(0, 5).map((url: string) => (
                    <a
                      key={url}
                      href={url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="block text-xs text-blue-500 hover:underline truncate"
                    >
                      {new URL(url).hostname}
                    </a>
                  ))}
                </div>
              </div>
            )}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

// Main CommandPalette Component
export function CommandPalette({ onClose }: { onClose?: () => void }) {
  const { state, actions } = useCommandEngine();
  const inputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [showPreview, setShowPreview] = useState(false);
  const lastInputTimeRef = useRef(Date.now());

  // Physics-based input animation based on typing velocity
  const inputScale = useMotionValue(1);
  const inputGlow = useMotionValue(0);
  const scaleSpring = useSpring(inputScale, { stiffness: 400, damping: 30 });
  const glowSpring = useSpring(inputGlow, { stiffness: 300, damping: 25 });
  const glowOpacity = useTransform(glowSpring, [0, 1], [0, 0.3]);

  // Track typing velocity
  useEffect(() => {
    if (state.input.length > 0) {
      const now = Date.now();
      const timeDelta = now - lastInputTimeRef.current;
      const velocity = timeDelta > 0 ? 1000 / timeDelta : 0;
      lastInputTimeRef.current = now;

      // Trigger input animation
      inputScale.set(1 + velocity * 0.01);
      inputGlow.set(Math.min(velocity * 0.1, 1));
    } else {
      inputScale.set(1);
      inputGlow.set(0);
    }
  }, [state.input, inputScale, inputGlow]);

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        actions.selectNext();
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        actions.selectPrev();
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (state.results[state.selection]) {
          actions.navigateToResult(state.selection);
        }
      } else if (e.key === 'Escape') {
        if (state.input) {
          actions.clear();
        } else if (onClose) {
          onClose();
        }
        inputRef.current?.blur();
      } else if (e.key === 'Tab' && state.results[state.selection]) {
        e.preventDefault();
        setShowPreview(!showPreview);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [state.selection, state.results, actions, showPreview]);

  // Auto-focus input
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Get suggestions for ghost text
  const suggestions = useRef<string[]>([]);
  useEffect(() => {
    if (state.input.length >= 2) {
      // Simple suggestion based on popular/recent
      const all = [
        ...state.popular.map(p => p.url),
        ...state.recent.map(r => r.query),
      ];
      suggestions.current = Array.from(new Set(all)).slice(0, 10);
    } else {
      suggestions.current = [];
    }
  }, [state.input, state.popular, state.recent]);

  const selectedResult = state.results[state.selection] || null;

  return (
    <div 
      className="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center p-4 z-[300]"
      onClick={(e) => {
        // Close on backdrop click
        if (e.target === e.currentTarget && onClose) {
          onClose();
        }
      }}
    >
      <motion.div
        ref={containerRef}
        initial={{ opacity: 0, scale: 0.95, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 20 }}
        transition={{
          type: 'spring',
          stiffness: 300,
          damping: 30,
        }}
        className="relative w-full max-w-2xl bg-white dark:bg-gray-800 rounded-xl shadow-xl border border-gray-200 dark:border-gray-700 overflow-hidden"
        style={{ maxHeight: '600px' }}
      >
        {/* Input Container with Physics */}
        <div className="relative p-4 border-b border-gray-200 dark:border-gray-700">
          <motion.div
            style={{
              scale: scaleSpring,
              boxShadow: `0 0 ${glowOpacity.get() * 40}px rgba(59, 130, 246, ${glowOpacity.get()})`,
            }}
            className="relative"
          >
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400 dark:text-gray-500 pointer-events-none" />
            <input
              ref={inputRef}
              type="text"
              value={state.input}
              onChange={(e) => actions.setInput(e.target.value)}
              placeholder="Search your browsing history..."
              className="w-full pl-12 pr-10 py-3 bg-transparent text-base text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none"
            />
            <GhostInput input={state.input} suggestions={suggestions.current} />
            {state.input && (
              <motion.button
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                onClick={actions.clear}
                className="absolute right-4 top-1/2 -translate-y-1/2 p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              >
                <X className="w-4 h-4 text-gray-600 dark:text-gray-400" />
              </motion.button>
            )}
          </motion.div>

          {/* Status Bar */}
          {(state.isLoading || state.queryTime !== null || state.error) && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              className="mt-2 text-xs text-gray-600 dark:text-gray-400 flex items-center gap-3"
            >
              {state.isLoading && (
                <span className="flex items-center gap-2">
                  <motion.div
                    animate={{ rotate: 360 }}
                    transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
                  >
                    <Sparkles className="w-3 h-3" />
                  </motion.div>
                  Searching...
                </span>
              )}
              {state.queryTime !== null && !state.isLoading && (
                <span>Found {state.results.length} results in {state.queryTime}ms</span>
              )}
              {state.error && (
                <span className="flex items-center gap-1 text-red-500">
                  <AlertCircle className="w-3 h-3" />
                  {state.error}
                </span>
              )}
            </motion.div>
          )}
        </div>

        {/* Results Container with LayoutGroup for FLIP animations */}
        <div className="relative overflow-y-auto" style={{ maxHeight: '500px' }}>
          <LayoutGroup>
            <AnimatePresence mode="wait">
              {state.input.trim() ? (
                // Search Results
                state.results.length > 0 ? (
                  <motion.div
                    key="results"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: durations.fast / 1000 }}
                  >
                    {state.results.map((result, index) => (
                      <ResultRow
                        key={`${result.url}-${index}`}
                        result={result}
                        index={index}
                        isSelected={index === state.selection}
                        onSelect={() => actions.selectIndex(index)}
                        onNavigate={() => actions.navigateToResult(index)}
                      />
                    ))}
                  </motion.div>
                ) : !state.isLoading ? (
                  <motion.div
                    key="empty"
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="p-8 text-center"
                  >
                    <p className="text-gray-600 dark:text-gray-400">No results found</p>
                    <p className="text-sm text-gray-500 dark:text-gray-500 mt-2">Try a different search term</p>
                  </motion.div>
                ) : null
              ) : (
                // Initial State: Popular + Recent
                <motion.div
                  key="initial"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="p-4"
                >
                  {state.recent.length > 0 && (
                    <div className="mb-6">
                      <h3 className="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase tracking-wide mb-3">
                        Recent Searches
                      </h3>
                      <div className="space-y-1">
                        {state.recent.slice(0, 5).map((item, index) => (
                          <motion.button
                            key={item.query}
                            initial={{ opacity: 0, x: -10 }}
                            animate={{ opacity: 1, x: 0 }}
                            transition={{ delay: index * 0.05 }}
                            onClick={() => {
                              actions.setInput(item.query);
                              actions.search(item.query);
                            }}
                            className="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-sm text-gray-900 dark:text-gray-100 flex items-center justify-between"
                          >
                            <span className="flex items-center gap-2">
                              <Clock className="w-4 h-4 text-gray-500 dark:text-gray-500" />
                              {item.query}
                            </span>
                            {item.resultCount && (
                              <span className="text-xs text-gray-500 dark:text-gray-500">
                                {item.resultCount} results
                              </span>
                            )}
                          </motion.button>
                        ))}
                      </div>
                    </div>
                  )}

                  {state.popular.length > 0 && (
                    <div>
                      <h3 className="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase tracking-wide mb-3 flex items-center gap-2">
                        <TrendingUp className="w-4 h-4" />
                        Most Visited
                      </h3>
                      <div className="space-y-1">
                        {state.popular.slice(0, 10).map((item, index) => (
                          <motion.button
                            key={item.url}
                            initial={{ opacity: 0, x: -10 }}
                            animate={{ opacity: 1, x: 0 }}
                            transition={{ delay: index * 0.05 }}
                            onClick={() => {
                              window.open(item.url, '_blank', 'noopener,noreferrer');
                              // Add to recent
                              recentSearches.add(item.url, 1);
                            }}
                            className="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-sm text-gray-900 dark:text-gray-100 flex items-center justify-between"
                          >
                            <span className="truncate">{new URL(item.url).hostname}</span>
                            <span className="text-xs text-gray-500 dark:text-gray-500 ml-2">
                              {item.visits} visits
                            </span>
                          </motion.button>
                        ))}
                      </div>
                    </div>
                  )}
                </motion.div>
              )}
            </AnimatePresence>
          </LayoutGroup>
        </div>

        {/* Preview Pane */}
        <PreviewPane result={selectedResult} isVisible={showPreview && !!selectedResult} />
      </motion.div>
    </div>
  );
}
