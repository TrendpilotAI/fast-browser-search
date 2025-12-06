import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { api, SearchResult } from '../lib/api';
import debounce from 'lodash/debounce';

export interface BrowserFilter {
  chrome: boolean;
  safari: boolean;
  arc: boolean;
  comet: boolean;
  genspark: boolean;
  thorium: boolean;
  gmail: boolean;
}

export const useMainWindowSearch = () => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [browserFilters, setBrowserFilters] = useState<BrowserFilter>({
    chrome: false,
    safari: false,
    arc: false,
    comet: false,
    genspark: false,
    thorium: false,
    gmail: false,
  });
  
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
        const popular = await api.getPopular();
        
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

  // Search Logic with browser filters
  const performSearch = useMemo(
    () => debounce(async (q: string, filters: BrowserFilter) => {
      if (!q.trim()) return;

      const cacheKey = `${q}:${JSON.stringify(filters)}`;
      if (cache.current.has(cacheKey)) {
        setResults(cache.current.get(cacheKey)!);
        return;
      }

      setIsLoading(true);
      try {
        // Build browser filter array for API
        const activeFilters = Object.entries(filters)
          .filter(([_, active]) => active)
          .map(([browser]) => browser.charAt(0).toUpperCase() + browser.slice(1));

        const [searchRes, suggestRes] = await Promise.all([
          api.search(q, { browsers: activeFilters.length > 0 ? activeFilters : undefined }),
          api.suggest(q)
        ]);

        // Results are already filtered by backend if browsers array is provided
        let filteredResults = searchRes.results;

        const mappedResults: SearchResult[] = filteredResults.map((r: any) => ({
            id: r.url,
            url: r.url,
            title: r.title || r.url,
            description: r.summary || r.description || r.url,
            last_visit: r.visit_time,
            visit_count: r.visit_count,
            source: (r.browser_source ? r.browser_source.toLowerCase() : 'web') as any,
            score: r.relevance_score,
            tags: r.key_topics || r.tags || [],
            
            // Pass through semantic fields
            clean_site_name: r.clean_site_name,
            site_category: r.site_category,
            key_topics: r.key_topics,
            summary: r.summary,
        }));

        setResults(mappedResults);
        setSuggestions(suggestRes);
        
        cache.current.set(cacheKey, mappedResults);
      } catch (e) {
        console.error(e);
      } finally {
        setIsLoading(false);
      }
    }, 200), // 200ms debounce for main window
    []
  );

  // Effect to trigger search
  useEffect(() => {
    if (query) {
        performSearch(query, browserFilters);
    } else {
        // Clear results or show initial state handled by other effect
    }
    setSelectedIndex(0);
  }, [query, browserFilters, performSearch]);

  // Toggle browser filter
  const toggleBrowserFilter = useCallback((browser: keyof BrowserFilter) => {
    setBrowserFilters(prev => ({
      ...prev,
      [browser]: !prev[browser]
    }));
  }, []);

  // Action: Select/Execute
  const addToHistory = (q: string) => {
    const newHistory = [q, ...recentQueries.filter(x => x !== q)].slice(0, 10);
    setRecentQueries(newHistory);
    localStorage.setItem('fbs_recent_queries', JSON.stringify(newHistory));
  };

  // Keyboard Navigation
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
    addToHistory,
    browserFilters,
    toggleBrowserFilter
  };
};

