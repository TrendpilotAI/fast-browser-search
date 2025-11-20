import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { api, SearchResult } from '../lib/api';
import { debounce } from 'lodash';

export const useCommandEngine = () => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  
  // Cache for instant back-navigation: Query -> Results[]
  const cache = useRef<Map<string, SearchResult[]>>(new Map());
  
  // Local History (persisted)
  const [recentQueries, setRecentQueries] = useState<string[]>(() => {
    try {
      const saved = localStorage.getItem('fbs_recent_queries');
      return saved ? JSON.parse(saved) : [];
    } catch (e) {
      return [];
    }
  });

  // Initial Load: Popular items + Recent Queries
  useEffect(() => {
    const loadInitial = async () => {
      setIsLoading(true);
      try {
        // Parallel fetch if you want more data types
        const popular = await api.getPopular();
        
        // Convert recent queries to "results" for the UI
        const historyResults: SearchResult[] = recentQueries.map((q, i) => ({
          id: `hist-${i}`,
          url: '',
          title: q,
          description: 'Recent Search',
          source: 'history',
          visit_count: 0,
          tags: ['recent']
        }));

        setResults([...historyResults, ...popular]);
      } catch (e) {
        console.error("Failed to load initial data", e);
      } finally {
        setIsLoading(false);
      }
    };

    if (!query) loadInitial();
  }, [query, recentQueries]);

  // Search Logic
  const performSearch = useMemo(
    () => debounce(async (q: string) => {
      if (!q.trim()) return;

      // Check cache first
      if (cache.current.has(q)) {
        setResults(cache.current.get(q)!);
        return;
      }

      setIsLoading(true);
      try {
        // Run search and suggestions in parallel
        const [searchRes, suggestRes] = await Promise.all([
          api.search(q),
          api.suggest(q)
        ]);

        const mappedResults: SearchResult[] = searchRes.results.map(r => ({
            ...r,
            source: r.source || 'web', // Fallback
            tags: r.tags || []
        }));

        setResults(mappedResults);
        setSuggestions(suggestRes);
        
        // Cache result
        cache.current.set(q, mappedResults);
      } catch (e) {
        console.error(e);
      } finally {
        setIsLoading(false);
      }
    }, 20), // 20ms debounce for "instant" feel
    []
  );

  // Effect to trigger search
  useEffect(() => {
    if (query) {
        performSearch(query);
    } else {
        // Clear results or show initial state handled by other effect
    }
    // Reset selection on query change
    setSelectedIndex(0);
  }, [query, performSearch]);

  // Action: Select/Execute
  const addToHistory = (q: string) => {
    const newHistory = [q, ...recentQueries.filter(x => x !== q)].slice(0, 10);
    setRecentQueries(newHistory);
    localStorage.setItem('fbs_recent_queries', JSON.stringify(newHistory));
  };

  // Keyboard Navigation Physics
  const handleNavigation = useCallback((e: React.KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
          e.preventDefault();
          setSelectedIndex(prev => Math.min(prev + 1, results.length - 1));
      } else if (e.key === 'ArrowUp') {
          e.preventDefault();
          setSelectedIndex(prev => Math.max(prev - 1, 0));
      }
  }, [results.length]);

  return {
    query,
    setQuery,
    results,
    suggestions,
    isLoading,
    selectedIndex,
    setSelectedIndex,
    handleNavigation,
    addToHistory
  };
};

