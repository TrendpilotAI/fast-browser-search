import axios from 'axios';

const API_BASE = 'http://localhost:3000/api';

export interface SearchResult {
  id: string; // mapped from url or specialized ID
  url: string;
  title: string;
  description?: string;
  last_visit?: string;
  visit_count?: number;
  source: 'chrome' | 'safari' | 'arc' | 'gmail' | 'history' | 'web'; 
  score?: number;
  tags?: string[];
  
  // New Semantic Fields
  clean_site_name?: string;
  site_category?: string;
  key_topics?: string[];
  summary?: string;
}

export interface SearchResponse {
  results: SearchResult[];
  total: number;
  query_time_ms: number;
  semantic?: boolean;
}

export interface Suggestion {
  text: string;
  type: 'query' | 'url' | 'domain';
}

export const api = {
  /**
   * Perform a semantic search (or standard search if semantic is toggled off)
   */
  search: async (query: string, useSemantic = true): Promise<SearchResponse> => {
    const { data } = await axios.post<SearchResponse>(`${API_BASE}/semantic/search`, {
      query,
      limit: 20,
      use_semantic: useSemantic,
    });
    return data;
  },

  /**
   * Get autocomplete suggestions
   */
  suggest: async (query: string): Promise<string[]> => {
    const { data } = await axios.get<{ suggestions: string[] }>(`${API_BASE}/suggest`, {
      params: { query }
    });
    return data.suggestions;
  },

  /**
   * Get popular items for the initial view
   */
  getPopular: async (): Promise<SearchResult[]> => {
    const { data } = await axios.get<{ popular: { url: string, visits: number }[] }>(`${API_BASE}/popular`);
    
    // Transform backend generic format to UI SearchResult
    return data.popular.map((p, idx) => ({
      id: `pop-${idx}`,
      url: p.url,
      title: new URL(p.url).hostname, // Fallback title
      description: `Visited ${p.visits} times`,
      source: 'history',
      visit_count: p.visits,
      tags: ['popular']
    }));
  },

  /**
   * Record a visit (optional, if backend supports tracking clicks)
   */
  recordVisit: async (_url: string) => {
    // Implement if backend has a specific endpoint for boosting relevance
    // await axios.post(`${API_BASE}/visit`, { url: _url });
  }
};
