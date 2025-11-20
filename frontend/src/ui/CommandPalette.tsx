import React, { useState, useEffect, useRef } from 'react';
import { 
  Search, Mail, Globe, ArrowRight, Clock, 
  Cpu, Sparkles, LayoutGrid, HelpCircle
} from 'lucide-react';
import { motion, AnimatePresence, LayoutGroup } from 'framer-motion';
import { tokens } from './tokens';
import { useCommandEngine } from '../hooks/useCommandEngine';
import { SearchResult } from '../lib/api';
import clsx from 'clsx';

// --- Icons & Visuals ---

const SourceIcon = ({ source }: { source: string }) => {
  switch (source) {
    case 'gmail': return <Mail size={14} className="text-red-400" />;
    case 'chrome': return <Globe size={14} className="text-blue-400" />;
    case 'arc': return <Sparkles size={14} className="text-pink-400" />;
    case 'safari': return <Globe size={14} className="text-blue-300" />;
    case 'history': return <Clock size={14} className="text-text-muted" />;
    default: return <Globe size={14} className="text-emerald-400" />;
  }
};

// --- Sub-Components ---

const ResultRow = ({ 
  item, 
  selected, 
  onClick, 
  onMouseEnter 
}: { 
  item: SearchResult, 
  selected: boolean, 
  onClick: () => void,
  onMouseEnter: () => void
}) => {
  return (
    <motion.div 
      layout
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{ duration: 0.15 }}
      onClick={onClick}
      onMouseEnter={onMouseEnter}
      className="relative h-[56px] mx-2 px-3 rounded-[12px] flex items-center gap-4 cursor-pointer z-10 group"
    >
      {/* Active State Background (Morphing) */}
      {selected && (
        <motion.div
          layoutId="highlight"
          className="absolute inset-0 bg-surface-active rounded-[12px] shadow-sm z-[-1] border border-border-subtle/50"
          transition={{ 
            type: "spring", 
            stiffness: 400, 
            damping: 30 
          }}
        />
      )}

      {/* Icon */}
      <div className={clsx(
        "w-9 h-9 rounded-xl flex items-center justify-center border transition-all duration-200 shadow-sm",
        selected ? "bg-bg-elevated border-border-focus scale-105" : "bg-surface border-border-subtle group-hover:border-text-muted"
      )}>
        <SourceIcon source={item.source} />
      </div>

      {/* Text Content */}
      <div className="flex-1 min-w-0 flex flex-col justify-center gap-0.5">
        <div className="flex items-center gap-2">
          <span className={clsx(
            "text-[14px] truncate transition-colors",
            selected ? "text-text-primary font-medium" : "text-text-secondary"
          )}>
            {item.title || item.url}
          </span>
          
          {/* Tags */}
          {item.tags?.map(tag => (
            <span key={tag} className="hidden sm:inline-block text-[10px] px-1.5 py-0.5 rounded-md bg-bg-primary border border-border-subtle text-text-muted uppercase tracking-wider font-medium">
              {tag}
            </span>
          ))}
        </div>
        
        <div className="flex items-center gap-1.5 text-[11px] text-text-muted">
          <span className={clsx(selected ? "text-text-secondary" : "")}>
            {item.source === 'history' ? 'Previous Search' : `From ${item.source}`}
          </span>
          <span>•</span>
          <span className="truncate max-w-[300px]">{item.description || item.url}</span>
        </div>
      </div>

      {/* Explicit Action Button (Visible on Select) */}
      <AnimatePresence>
        {selected && (
          <motion.div 
            initial={{ opacity: 0, x: 10 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 5 }}
            className="flex items-center gap-3 pr-1"
          >
             <div className="hidden md:flex flex-col items-end text-[10px] text-text-muted leading-tight">
                <span className="font-medium text-text-secondary">Open</span>
                <span>↵ Enter</span>
             </div>
             <div className="w-7 h-7 rounded-lg flex items-center justify-center bg-accent-primary text-white shadow-lg shadow-accent-primary/20">
                <ArrowRight size={14} />
             </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
};

// --- Main Component ---

export const CommandPalette = () => {
  const { 
    query, setQuery, results, suggestions, isLoading, 
    selectedIndex, setSelectedIndex, handleNavigation, addToHistory 
  } = useCommandEngine();
  
  const [isOpen, setIsOpen] = useState(true);
  const [showPreview, setShowPreview] = useState(false);
  const [ghostText, setGhostText] = useState('');
  
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Toggle & Keyboard Trap
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsOpen(prev => !prev);
      }
      if (e.key === 'Escape') {
        if (query) setQuery('');
        else setIsOpen(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [query, setQuery]);

  // Ghost Text Calculation
  useEffect(() => {
    if (!query || suggestions.length === 0) {
      setGhostText('');
      return;
    }
    const bestMatch = suggestions[0]; 
    if (bestMatch.toLowerCase().startsWith(query.toLowerCase())) {
       setGhostText(bestMatch.slice(query.length));
    } else {
       setGhostText('');
    }
  }, [query, suggestions]);

  // Tab Handler
  const handleKeyDown = (e: React.KeyboardEvent) => {
      if (e.key === 'Tab') {
          e.preventDefault();
          if (ghostText) {
              setQuery(query + ghostText);
          } else {
              setShowPreview(prev => !prev);
          }
      } else if (e.key === 'Enter') {
          e.preventDefault();
          const item = results[selectedIndex];
          if (item) {
             if (item.source === 'history' && !item.url) {
                 setQuery(item.title);
             } else {
                 addToHistory(query || item.title);
                 window.open(item.url, '_blank');
                 setIsOpen(false);
             }
          }
      }
      handleNavigation(e);
  };

  if (!isOpen) return null;

  const selectedItem = results[selectedIndex];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm text-sans antialiased font-feature-default">
      <motion.div 
        initial={{ opacity: 0, scale: 0.95, y: 10 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 10 }}
        transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
        className="flex overflow-hidden bg-bg-elevated border border-border-subtle shadow-2xl ring-1 ring-white/5"
        style={{ 
          height: '600px', 
          width: showPreview ? '1080px' : '840px',
          borderRadius: tokens.radius.lg
        }}
      >
        {/* LEFT COLUMN (Main) */}
        <div className="flex-1 flex flex-col min-w-0 relative bg-gradient-to-b from-bg-elevated to-bg-primary">
            
          {/* Header Input */}
          <div className="h-[72px] border-b border-border-subtle flex items-center px-6 gap-5 shrink-0 relative z-20 bg-bg-elevated/80 backdrop-blur-md">
            <motion.div 
                animate={{ scale: isLoading ? 0.9 : 1 }}
                className={clsx("transition-colors duration-300 p-2 rounded-lg bg-surface border border-border-subtle", isLoading ? "text-accent-primary border-accent-primary/30" : "text-text-muted")}
            >
                {isLoading ? <Cpu size={20} className="animate-spin-slow" /> : <Search size={20} />}
            </motion.div>

            <div className="relative flex-1 h-full flex items-center group">
               {/* Ghost Overlay */}
               <div className="absolute inset-0 flex items-center pointer-events-none text-[20px] tracking-tight pl-1">
                  <span className="text-transparent whitespace-pre">{query}</span>
                  <span className="text-text-muted/30">{ghostText}</span>
               </div>
               
               <input
                 ref={inputRef}
                 autoFocus
                 value={query}
                 onChange={(e) => setQuery(e.target.value)}
                 onKeyDown={handleKeyDown}
                 className="w-full bg-transparent text-text-primary placeholder-text-muted/20 focus:outline-none text-[20px] font-medium tracking-tight pl-1 h-full"
                 placeholder="Type to search history..."
                 spellCheck={false}
               />
            </div>

            {/* Mode Badge */}
            {results.length > 0 && (
                <div className="flex items-center gap-2 px-3 py-1.5 bg-surface rounded-full border border-border-subtle">
                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.5)]" />
                    <span className="text-[11px] font-medium text-text-secondary">Semantic Search</span>
                </div>
            )}
          </div>

          {/* Results List */}
          <div className="flex-1 overflow-y-auto py-4 scrollbar-thin scrollbar-thumb-border-subtle hover:scrollbar-thumb-border-focus scrollbar-track-transparent" ref={listRef}>
            
            {/* Empty State Guidance */}
            {results.length === 0 && !isLoading && !query && (
                <div className="p-8 flex flex-col items-center justify-center h-full text-center opacity-0 animate-in fade-in duration-500 slide-in-from-bottom-2">
                    <div className="w-16 h-16 rounded-2xl bg-surface border border-border-subtle flex items-center justify-center mb-6 shadow-xl">
                        <LayoutGrid size={32} className="text-accent-primary" strokeWidth={1.5} />
                    </div>
                    <h3 className="text-lg font-medium text-text-primary mb-2">Start searching</h3>
                    <p className="text-sm text-text-muted max-w-xs leading-relaxed">
                        Type anything to search across Chrome, Safari, Arc, and Gmail instantly.
                    </p>
                    
                    <div className="mt-8 grid grid-cols-2 gap-3 w-full max-w-md">
                        {['Docs', 'Meetings', 'Design', 'Invoices'].map(tag => (
                            <button key={tag} onClick={() => setQuery(tag)} className="p-3 rounded-lg bg-surface border border-border-subtle hover:bg-surface-hover text-sm text-text-secondary transition-colors text-left flex items-center gap-2">
                                <Search size={14} className="text-text-muted" />
                                {tag}
                            </button>
                        ))}
                    </div>
                </div>
            )}

            <LayoutGroup>
                <AnimatePresence mode='popLayout'>
                    {results.map((item, index) => (
                        <ResultRow 
                            key={item.id || index} 
                            item={item} 
                            selected={index === selectedIndex}
                            onClick={() => {
                                setSelectedIndex(index);
                            }}
                            onMouseEnter={() => setSelectedIndex(index)}
                        />
                    ))}
                </AnimatePresence>
            </LayoutGroup>
            
            {results.length === 0 && !isLoading && query && (
                <div className="flex flex-col items-center justify-center h-40 text-text-muted gap-3 opacity-50">
                    <LayoutGrid size={32} strokeWidth={1.5} />
                    <p className="text-sm">No matches found for "{query}"</p>
                </div>
            )}
          </div>

          {/* Explicit Footer Hints */}
          <div className="h-[48px] border-t border-border-subtle bg-surface/80 flex items-center px-6 justify-between text-[11px] text-text-muted select-none backdrop-blur-md z-20">
             <div className="flex gap-6">
                <div className="flex items-center gap-2 group cursor-pointer hover:text-text-primary transition-colors">
                    <kbd className="font-sans bg-surface border border-border-subtle px-1.5 py-0.5 rounded-[4px] min-w-[20px] text-center shadow-sm group-hover:border-text-muted transition-colors">↵</kbd> 
                    <span>Open</span>
                </div>
                <div className="flex items-center gap-2 group cursor-pointer hover:text-text-primary transition-colors">
                    <kbd className="font-sans bg-surface border border-border-subtle px-1.5 py-0.5 rounded-[4px] min-w-[20px] text-center shadow-sm group-hover:border-text-muted transition-colors">⇥</kbd> 
                    <span>Preview</span>
                </div>
                <div className="flex items-center gap-2 group cursor-pointer hover:text-text-primary transition-colors">
                    <kbd className="font-sans bg-surface border border-border-subtle px-1.5 py-0.5 rounded-[4px] min-w-[20px] text-center shadow-sm group-hover:border-text-muted transition-colors">↑↓</kbd> 
                    <span>Navigate</span>
                </div>
             </div>
             <div className="flex items-center gap-2 opacity-50 hover:opacity-100 transition-opacity">
                <HelpCircle size={12} />
                <span>Press ? for help</span>
             </div>
          </div>
        </div>

        {/* RIGHT COLUMN (Preview) */}
        <AnimatePresence>
            {showPreview && selectedItem && (
                <motion.div 
                    initial={{ width: 0, opacity: 0 }}
                    animate={{ width: 420, opacity: 1 }}
                    exit={{ width: 0, opacity: 0 }}
                    transition={{ type: "spring", stiffness: 300, damping: 30 }}
                    className="border-l border-border-subtle bg-surface flex flex-col relative z-10 shadow-2xl"
                >
                    <div className="p-8 flex flex-col h-full">
                         {/* Preview Header */}
                         <div className="flex items-start gap-5 mb-8">
                            <div className="w-14 h-14 rounded-[18px] bg-surface-hover border border-border-subtle flex items-center justify-center shrink-0 shadow-sm">
                                <SourceIcon source={selectedItem.source} />
                            </div>
                            <div className="min-w-0 pt-1">
                                <h3 className="text-[18px] font-semibold text-text-primary leading-snug mb-1.5 break-words">
                                    {selectedItem.title}
                                </h3>
                                <div className="flex items-center gap-2">
                                    <span className="px-2 py-0.5 rounded-full bg-bg-primary border border-border-subtle text-[10px] font-medium text-text-secondary uppercase tracking-wide">
                                        {selectedItem.source}
                                    </span>
                                    <span className="text-[12px] text-text-muted font-mono">
                                        {selectedItem.last_visit || 'Just now'}
                                    </span>
                                </div>
                            </div>
                         </div>

                         {/* Preview Content Mock */}
                         <div className="flex-1 bg-bg-primary/50 rounded-xl border border-border-subtle p-6 overflow-y-auto relative">
                            <div className="absolute top-0 left-0 right-0 h-4 bg-gradient-to-b from-bg-primary/50 to-transparent pointer-events-none" />
                            
                            <div className="space-y-4">
                                <div className="h-2 w-3/4 bg-surface-active rounded-full animate-pulse" />
                                <div className="h-2 w-full bg-surface-active rounded-full animate-pulse delay-75" />
                                <div className="h-2 w-5/6 bg-surface-active rounded-full animate-pulse delay-150" />
                                <div className="h-2 w-4/5 bg-surface-active rounded-full animate-pulse delay-200" />
                                <div className="h-2 w-full bg-surface-active rounded-full animate-pulse delay-300 opacity-50" />
                            </div>

                            <div className="mt-8 pt-8 border-t border-border-subtle/50">
                                <h4 className="text-[11px] uppercase tracking-wider text-text-muted mb-4 font-medium flex items-center gap-2">
                                    <LayoutGrid size={12} /> Metadata
                                </h4>
                                <div className="grid grid-cols-2 gap-4">
                                    <div className="bg-surface p-3 rounded-xl border border-border-subtle">
                                        <span className="text-[10px] text-text-muted block mb-1">Visits</span>
                                        <span className="text-[14px] text-text-primary font-mono font-medium">{selectedItem.visit_count || 1}</span>
                                    </div>
                                    <div className="bg-surface p-3 rounded-xl border border-border-subtle">
                                        <span className="text-[10px] text-text-muted block mb-1">Relevance Score</span>
                                        <div className="flex items-center gap-2">
                                            <div className="h-1.5 flex-1 bg-bg-primary rounded-full overflow-hidden">
                                                <div className="h-full w-3/4 bg-accent-primary rounded-full" />
                                            </div>
                                            <span className="text-[14px] text-text-primary font-mono font-medium">98%</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                         </div>

                         {/* Actions */}
                         <div className="mt-8 pt-4 border-t border-border-subtle flex flex-col gap-3">
                            <button 
                                className="w-full h-11 bg-accent-primary text-white rounded-xl text-[14px] font-medium hover:bg-accent-primary/90 transition-all shadow-lg shadow-accent-primary/25 hover:shadow-accent-primary/40 hover:-translate-y-0.5 flex items-center justify-center gap-2 active:scale-95 active:translate-y-0"
                                onClick={() => window.open(selectedItem.url, '_blank')}
                            >
                                <span>Open in Browser</span>
                                <ArrowRight size={16} />
                            </button>
                            <div className="grid grid-cols-2 gap-3">
                                <button className="h-9 rounded-lg bg-surface border border-border-subtle text-text-secondary text-xs hover:bg-surface-hover hover:text-text-primary transition-colors font-medium">
                                    Copy Link
                                </button>
                                <button className="h-9 rounded-lg bg-surface border border-border-subtle text-text-secondary text-xs hover:bg-surface-hover hover:text-text-primary transition-colors font-medium">
                                    Share
                                </button>
                            </div>
                         </div>
                    </div>
                </motion.div>
            )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
};
