/**
 * useCommandEngine Hook
 * Master-class logic layer with hybrid state, smart data fusion, and instant cache
 */

import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { api, recentSearches } from './api';
import type { SearchResult, PopularUrl, RecentSearch } from './api';

// LRU Cache for instant back-navigation
class LRUCache<K, V> {
  private capacity: number;
  private cache: Map<K, V>;

  constructor(capacity: number = 50) {
    this.capacity = capacity;
    this.cache = new Map();
  }

  get(key: K): V | undefined {
    if (!this.cache.has(key)) return undefined;
    const value = this.cache.get(key);
    if (value === undefined) return undefined;
    // Move to end (most recently used)
    this.cache.delete(key);
    this.cache.set(key, value);
    return value;
  }

  set(key: K, value: V): void {
    if (this.cache.has(key)) {
      this.cache.delete(key);
    } else if (this.cache.size >= this.capacity) {
      // Remove least recently used (first item)
      const firstKey = this.cache.keys().next().value;
      if (firstKey !== undefined) {
        this.cache.delete(firstKey);
      }
    }
    this.cache.set(key, value);
  }

  clear(): void {
    this.cache.clear();
  }

  size(): number {
    return this.cache.size;
  }
}

export interface CommandState {
  input: string;
  results: SearchResult[];
  popular: PopularUrl[];
  recent: RecentSearch[];
  selection: number;
  isLoading: boolean;
  error: string | null;
  queryTime: number | null;
}

export interface CommandActions {
  setInput: (value: string) => void;
  search: (query: string) => Promise<void>;
  selectNext: () => void;
  selectPrev: () => void;
  selectIndex: (index: number) => void;
  clear: () => void;
  navigateToResult: (index: number) => void;
}

export interface UseCommandEngineReturn {
  state: CommandState;
  actions: CommandActions;
}

export function useCommandEngine(): UseCommandEngineReturn {
  // Hybrid State
  const [input, setInput] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [popular, setPopular] = useState<PopularUrl[]>([]);
  const [recent, setRecent] = useState<RecentSearch[]>([]);
  const [selection, setSelection] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [queryTime, setQueryTime] = useState<number | null>(null);

  // Instant Cache: LRU Map for query -> results
  const cacheRef = useRef(new LRUCache<string, SearchResponse>());
  
  // Search history for back-navigation
  const historyRef = useRef<string[]>([]);
  const historyIndexRef = useRef(-1);

  interface SearchResponse {
    results: SearchResult[];
    queryTime: number;
  }

  // Smart Data Fusion: Parallel fetch popular + recent on mount
  useEffect(() => {
    let cancelled = false;

    const loadInitialData = async () => {
      try {
        // Parallel fetch
        const [popularData, recentData] = await Promise.all([
          api.getPopular(20),
          Promise.resolve(recentSearches.get()),
        ]);

        if (!cancelled) {
          setPopular(popularData);
          setRecent(recentData);
        }
      } catch (err) {
        if (!cancelled) {
          console.error('Failed to load initial data:', err);
        }
      }
    };

    loadInitialData();

    return () => {
      cancelled = true;
    };
  }, []);

  // Search function with cache and history
  const search = useCallback(async (query: string) => {
    if (!query.trim()) {
      setResults([]);
      setQueryTime(null);
      return;
    }

    // Check cache first
    const cached = cacheRef.current.get(query);
    if (cached) {
      setResults(cached.results);
      setQueryTime(cached.queryTime);
      setSelection(0);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const response = await api.search({
        query,
        limit: 50,
        use_semantic: true, // Use semantic search by default
      });

      const searchResponse: SearchResponse = {
        results: response.results,
        queryTime: response.query_time_ms,
      };

      // Cache the results
      cacheRef.current.set(query, searchResponse);

      // Update state
      setResults(response.results);
      setQueryTime(response.query_time_ms);
      setSelection(0);

      // Add to recent searches
      recentSearches.add(query, response.total);

      // Update history for back-navigation
      historyRef.current = historyRef.current.slice(0, historyIndexRef.current + 1);
      historyRef.current.push(query);
      historyIndexRef.current = historyRef.current.length - 1;

      // Reload recent searches
      setRecent(recentSearches.get());
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Search failed';
      setError(errorMessage);
      setResults([]);
      setQueryTime(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Debounced search on input change
  const searchTimeoutRef = useRef<number | undefined>(undefined);
  useEffect(() => {
    if (searchTimeoutRef.current !== undefined) {
      clearTimeout(searchTimeoutRef.current);
    }

    if (input.trim()) {
      searchTimeoutRef.current = window.setTimeout(() => {
        search(input);
      }, 300) as unknown as number; // 300ms debounce
    } else {
      setResults([]);
      setQueryTime(null);
    }

    return () => {
      if (searchTimeoutRef.current !== undefined) {
        clearTimeout(searchTimeoutRef.current);
      }
    };
  }, [input, search]);

  // Selection navigation
  const selectNext = useCallback(() => {
    setSelection((prev) => {
      const maxIndex = results.length - 1;
      return prev < maxIndex ? prev + 1 : prev;
    });
  }, [results.length]);

  const selectPrev = useCallback(() => {
    setSelection((prev) => (prev > 0 ? prev - 1 : 0));
  }, []);

  const selectIndex = useCallback((index: number) => {
    if (index >= 0 && index < results.length) {
      setSelection(index);
    }
  }, [results.length]);

  // Navigate to result (open URL)
  const navigateToResult = useCallback((index: number) => {
    if (index >= 0 && index < results.length) {
      const result = results[index];
      window.open(result.url, '_blank', 'noopener,noreferrer');
      
      // Add to recent searches
      recentSearches.add(result.url, 1);
      setRecent(recentSearches.get());
    }
  }, [results]);

  // Clear function
  const clear = useCallback(() => {
    setInput('');
    setResults([]);
    setSelection(0);
    setError(null);
    setQueryTime(null);
    historyRef.current = [];
    historyIndexRef.current = -1;
  }, []);

  // Actions object
  const actions: CommandActions = useMemo(
    () => ({
      setInput,
      search,
      selectNext,
      selectPrev,
      selectIndex,
      clear,
      navigateToResult,
    }),
    [search, selectNext, selectPrev, selectIndex, clear, navigateToResult]
  );

  // State object
  const state: CommandState = useMemo(
    () => ({
      input,
      results,
      popular,
      recent,
      selection,
      isLoading,
      error,
      queryTime,
    }),
    [input, results, popular, recent, selection, isLoading, error, queryTime]
  );

  return { state, actions };
}
