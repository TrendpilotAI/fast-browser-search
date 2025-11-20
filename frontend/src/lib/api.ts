import { invoke } from '@tauri-apps/api/tauri';
import axios from 'axios';

const IS_TAURI = typeof window !== 'undefined' && '__TAURI__' in window;
const API_BASE = 'http://localhost:3002/api';

export type SearchResult = {
  url: string;
  title?: string;
  visit_time: string;
  visit_count: number;
  relevance_score: number;
  browser_source: string;
  domain: string;
  related_urls: string[];
  clean_site_name?: string;
  site_category?: string;
  key_topics?: string[];
  summary?: string;
};

export interface SearchResponse {
  results: SearchResult[];
  total: number;
  query_time_ms: number;
}

export const api = {
  search: async (query: string, browsers?: string[], limit = 50): Promise<SearchResponse> => {
    if (IS_TAURI) {
      const start = Date.now();
      const results = await invoke<SearchResult[]>('search', {
        query,
        limit,
        offset: 0,
        useSemantic: true
      });
      return {
        results,
        total: results.length,
        query_time_ms: Date.now() - start
      };
    } else {
      const response = await axios.post<any>(`${API_BASE}/search`, {
        query,
        browsers,
        limit
      });
      // Handle potentially different response structure from semantic API
      if (response.data.results) {
          return response.data;
      }
      return {
          results: response.data, // if it returns just array
          total: response.data.length || 0,
          query_time_ms: 0
      };
    }
  },

  suggest: async (query: string): Promise<string[]> => {
    if (IS_TAURI) {
      return invoke<string[]>('suggest', { query });
    } else {
      const response = await axios.get(`${API_BASE}/suggest`, { params: { query } });
      return response.data.suggestions;
    }
  },

  getPopular: async (limit = 20): Promise<{ url: string; visits: number }[]> => {
    if (IS_TAURI) {
      const results = await invoke<[string, number][]>('get_popular', { limit });
      return results.map(([url, visits]) => ({ url, visits }));
    } else {
      const response = await axios.get(`${API_BASE}/popular`);
      return response.data.popular;
    }
  },

  getDomains: async (): Promise<string[]> => {
    if (IS_TAURI) {
      return invoke<string[]>('get_domains');
    } else {
      const response = await axios.get(`${API_BASE}/domains`);
      return response.data.domains;
    }
  },

  index: async (): Promise<void> => {
    if (IS_TAURI) {
      await invoke('index_history');
    } else {
      await axios.post(`${API_BASE}/index`);
    }
  },

  connectGoogle: async (): Promise<string> => {
    if (IS_TAURI) {
      return invoke<string>('connect_google');
    } else {
      throw new Error("Google Auth only supported in Desktop App for now");
    }
  }
};
