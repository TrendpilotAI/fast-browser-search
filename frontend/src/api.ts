/**
 * API Layer with Rust Type Definitions
 * Strict type safety for all backend endpoints
 */

import axios from 'axios';

const API_BASE = 'http://localhost:3000/api';

// Rust-compatible types matching backend
export interface SearchResult {
  url: string;
  title?: string;
  visit_time: string;
  visit_count: number;
  relevance_score: number;
  browser_source: string;
  domain: string;
  related_urls: string[];
}

export interface SearchRequest {
  query: string;
  limit?: number;
  browsers?: string[];
  use_semantic?: boolean;
}

export interface SearchResponse {
  results: SearchResult[];
  total: number;
  query_time_ms: number;
}

export interface PopularUrl {
  url: string;
  visits: number;
  title?: string;
  domain: string;
}

export interface PopularResponse {
  popular: PopularUrl[];
}

export interface SuggestResponse {
  suggestions: string[];
}

export interface DomainsResponse {
  domains: string[];
}

export interface RelatedResponse {
  related: string[];
}

// API Client
class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE) {
    this.baseUrl = baseUrl;
  }

  /**
   * Search with optional semantic search
   */
  async search(request: SearchRequest): Promise<SearchResponse> {
    const response = await axios.post<SearchResponse>(
      `${this.baseUrl}/search`,
      request
    );
    return response.data;
  }

  /**
   * Semantic search (alternative endpoint)
   */
  async semanticSearch(request: SearchRequest): Promise<SearchResponse> {
    const response = await axios.post<SearchResponse>(
      `${this.baseUrl}/semantic/search`,
      { ...request, use_semantic: true }
    );
    return response.data;
  }

  /**
   * Get search suggestions
   */
  async getSuggestions(query: string): Promise<string[]> {
    if (query.length < 2) return [];
    const response = await axios.get<SuggestResponse>(
      `${this.baseUrl}/suggest`,
      { params: { query } }
    );
    return response.data.suggestions;
  }

  /**
   * Get popular URLs
   */
  async getPopular(limit: number = 20): Promise<PopularUrl[]> {
    const response = await axios.get<PopularResponse>(
      `${this.baseUrl}/popular`,
      { params: { limit } }
    );
    return response.data.popular;
  }

  /**
   * Get all indexed domains
   */
  async getDomains(): Promise<string[]> {
    const response = await axios.get<DomainsResponse>(
      `${this.baseUrl}/domains`
    );
    return response.data.domains;
  }

  /**
   * Get related URLs for a given URL
   */
  async getRelated(url: string, limit: number = 10): Promise<string[]> {
    const response = await axios.get<RelatedResponse>(
      `${this.baseUrl}/related`,
      { params: { url, limit } }
    );
    return response.data.related;
  }

  /**
   * Trigger re-indexing
   */
  async index(): Promise<void> {
    await axios.post(`${this.baseUrl}/index`);
  }

  /**
   * Health check
   */
  async health(): Promise<boolean> {
    try {
      const response = await axios.get(`${this.baseUrl.replace('/api', '')}/health`);
      return response.status === 200;
    } catch {
      return false;
    }
  }
}

// Singleton instance
export const api = new ApiClient();

// LocalStorage helpers for recent searches
const RECENT_SEARCHES_KEY = 'fast-browser-search:recent';
const MAX_RECENT = 10;

export interface RecentSearch {
  query: string;
  timestamp: number;
  resultCount?: number;
}

export const recentSearches = {
  /**
   * Get recent searches from localStorage
   */
  get(): RecentSearch[] {
    try {
      const stored = localStorage.getItem(RECENT_SEARCHES_KEY);
      if (!stored) return [];
      const items = JSON.parse(stored) as RecentSearch[];
      // Sort by timestamp, most recent first
      return items.sort((a, b) => b.timestamp - a.timestamp).slice(0, MAX_RECENT);
    } catch {
      return [];
    }
  },

  /**
   * Add a search to recent history
   */
  add(query: string, resultCount?: number): void {
    try {
      const items = this.get();
      // Remove duplicates
      const filtered = items.filter(item => item.query !== query);
      // Add new item at the beginning
      const updated = [
        { query, timestamp: Date.now(), resultCount },
        ...filtered,
      ].slice(0, MAX_RECENT);
      localStorage.setItem(RECENT_SEARCHES_KEY, JSON.stringify(updated));
    } catch {
      // Ignore localStorage errors
    }
  },

  /**
   * Clear recent searches
   */
  clear(): void {
    try {
      localStorage.removeItem(RECENT_SEARCHES_KEY);
    } catch {
      // Ignore localStorage errors
    }
  },
};
