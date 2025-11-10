import { useState, useCallback, useEffect } from 'react';
import { QueryClient, QueryClientProvider, useQuery, useMutation } from '@tanstack/react-query';
import axios from 'axios';
import { Search, Clock, Globe, Chrome, Compass, Sparkles, TrendingUp, RefreshCw, Globe2, Layers, Star } from 'lucide-react';
import { format } from 'date-fns';
import { clsx } from 'clsx';

const queryClient = new QueryClient();
const API_BASE = 'http://localhost:3000/api';

interface SearchResult {
  url: string;
  title?: string;
  visit_time: string;
  visit_count: number;
  relevance_score: number;
  browser_source: string;
  domain: string;
  related_urls: string[];
}

interface SearchResponse {
  results: SearchResult[];
  total: number;
  query_time_ms: number;
}

function SearchInterface() {
  const [query, setQuery] = useState('');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedBrowsers, setSelectedBrowsers] = useState<string[]>([]);
  const [wsConnection, setWsConnection] = useState<WebSocket | null>(null);
  const [realtimeResults, setRealtimeResults] = useState<SearchResult[]>([]);

  // Connect WebSocket for real-time search
  useEffect(() => {
    const ws = new WebSocket('ws://localhost:3000/ws');

    ws.onopen = () => {
      console.log('WebSocket connected');
      setWsConnection(ws);
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'search_results') {
        setRealtimeResults(data.results);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    return () => {
      ws.close();
    };
  }, []);

  // Search query
  const { data: searchData, isLoading, refetch } = useQuery({
    queryKey: ['search', searchTerm, selectedBrowsers],
    queryFn: async () => {
      if (!searchTerm) return null;
      const response = await axios.post<SearchResponse>(`${API_BASE}/search`, {
        query: searchTerm,
        limit: 50,
        browsers: selectedBrowsers.length > 0 ? selectedBrowsers : undefined,
      });
      return response.data;
    },
    enabled: !!searchTerm,
  });

  // Suggestions query
  const { data: suggestions } = useQuery({
    queryKey: ['suggestions', query],
    queryFn: async () => {
      if (query.length < 2) return [];
      const response = await axios.get(`${API_BASE}/suggest`, {
        params: { query }
      });
      return response.data.suggestions;
    },
    enabled: query.length >= 2,
  });

  // Popular URLs
  const { data: popularUrls } = useQuery({
    queryKey: ['popular'],
    queryFn: async () => {
      const response = await axios.get(`${API_BASE}/popular`);
      return response.data.popular;
    },
  });

  // Domains
  const { data: domains } = useQuery({
    queryKey: ['domains'],
    queryFn: async () => {
      const response = await axios.get(`${API_BASE}/domains`);
      return response.data.domains;
    },
  });

  // Index mutation
  const indexMutation = useMutation({
    mutationFn: async () => {
      const response = await axios.post(`${API_BASE}/index`);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries();
    },
  });

  const handleSearch = useCallback((e: React.FormEvent) => {
    e.preventDefault();
    setSearchTerm(query);

    // Send via WebSocket for real-time results
    if (wsConnection && wsConnection.readyState === WebSocket.OPEN) {
      wsConnection.send(JSON.stringify({
        query,
        limit: 50,
        browsers: selectedBrowsers.length > 0 ? selectedBrowsers : undefined,
      }));
    }
  }, [query, selectedBrowsers, wsConnection]);

  const toggleBrowser = (browser: string) => {
    setSelectedBrowsers(prev =>
      prev.includes(browser)
        ? prev.filter(b => b !== browser)
        : [...prev, browser]
    );
  };

  const getBrowserIcon = (browser: string) => {
    switch (browser.toLowerCase()) {
      case 'chrome': return <Chrome className="w-4 h-4" />;
      case 'safari': return <Globe2 className="w-4 h-4" />;
      case 'arc': return <Compass className="w-4 h-4" />;
      case 'comet': return <Star className="w-4 h-4" />;
      case 'thorium': return <Layers className="w-4 h-4" />;
      default: return <Globe className="w-4 h-4" />;
    }
  };

  const results = realtimeResults.length > 0 ? realtimeResults : searchData?.results || [];

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <Sparkles className="w-8 h-8 text-blue-500" />
              <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
                Fast Browser Search
              </h1>
            </div>
            <button
              onClick={() => indexMutation.mutate()}
              disabled={indexMutation.isPending}
              className="flex items-center space-x-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50"
            >
              <RefreshCw className={clsx('w-4 h-4', indexMutation.isPending && 'animate-spin')} />
              <span>{indexMutation.isPending ? 'Indexing...' : 'Re-index'}</span>
            </button>
          </div>
        </div>
      </header>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Search Bar */}
        <form onSubmit={handleSearch} className="mb-8">
          <div className="relative">
            <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search your browsing history..."
              className="w-full pl-12 pr-4 py-4 text-lg bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500"
              autoFocus
            />
          </div>

          {/* Suggestions */}
          {suggestions && suggestions.length > 0 && (
            <div className="mt-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg">
              {suggestions.map((suggestion: string) => (
                <button
                  key={suggestion}
                  type="button"
                  onClick={() => {
                    setQuery(suggestion);
                    setSearchTerm(suggestion);
                  }}
                  className="w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700"
                >
                  {suggestion}
                </button>
              ))}
            </div>
          )}
        </form>

        {/* Browser Filters */}
        <div className="mb-6 flex flex-wrap gap-2">
          {['Chrome', 'Safari', 'Arc', 'Comet', 'Genspark', 'Thorium'].map(browser => (
            <button
              key={browser}
              onClick={() => toggleBrowser(browser)}
              className={clsx(
                'flex items-center space-x-2 px-4 py-2 rounded-lg border transition-colors',
                selectedBrowsers.includes(browser)
                  ? 'bg-blue-500 text-white border-blue-500'
                  : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border-gray-300 dark:border-gray-600 hover:border-blue-500'
              )}
            >
              {getBrowserIcon(browser)}
              <span>{browser}</span>
            </button>
          ))}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Search Results */}
          <div className="lg:col-span-2">
            {isLoading && (
              <div className="flex justify-center py-12">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
              </div>
            )}

            {searchData && (
              <div className="mb-4 text-sm text-gray-600 dark:text-gray-400">
                Found {searchData.total} results in {searchData.query_time_ms}ms
              </div>
            )}

            <div className="space-y-4">
              {results.map((result, index) => (
                <div
                  key={`${result.url}-${index}`}
                  className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700 hover:shadow-lg transition-shadow"
                >
                  <a
                    href={result.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="block group"
                  >
                    <h3 className="text-lg font-semibold text-blue-600 dark:text-blue-400 group-hover:underline mb-1">
                      {result.title || result.url}
                    </h3>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-2 truncate">
                      {result.url}
                    </p>
                  </a>

                  <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-500">
                    <div className="flex items-center space-x-4">
                      <span className="flex items-center space-x-1">
                        <Clock className="w-3 h-3" />
                        <span>{format(new Date(result.visit_time), 'MMM d, yyyy')}</span>
                      </span>
                      <span className="flex items-center space-x-1">
                        {getBrowserIcon(result.browser_source)}
                        <span>{result.browser_source}</span>
                      </span>
                      <span>{result.visit_count} visits</span>
                    </div>
                  </div>

                  {result.related_urls && result.related_urls.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
                      <p className="text-xs text-gray-500 dark:text-gray-500 mb-1">Related:</p>
                      <div className="flex flex-wrap gap-1">
                        {result.related_urls.slice(0, 3).map(url => (
                          <a
                            key={url}
                            href={url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-xs px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
                          >
                            {new URL(url).hostname}
                          </a>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Popular URLs */}
            {popularUrls && popularUrls.length > 0 && (
              <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
                <h3 className="flex items-center space-x-2 text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                  <TrendingUp className="w-5 h-5" />
                  <span>Most Visited</span>
                </h3>
                <div className="space-y-2">
                  {popularUrls.slice(0, 5).map((item: any) => (
                    <a
                      key={item.url}
                      href={item.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="block text-sm text-blue-600 dark:text-blue-400 hover:underline"
                    >
                      <div className="truncate">{new URL(item.url).hostname}</div>
                      <div className="text-xs text-gray-500">{item.visits} visits</div>
                    </a>
                  ))}
                </div>
              </div>
            )}

            {/* Top Domains */}
            {domains && domains.length > 0 && (
              <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
                <h3 className="flex items-center space-x-2 text-lg font-semibold mb-4 text-gray-900 dark:text-white">
                  <Globe className="w-5 h-5" />
                  <span>Top Domains</span>
                </h3>
                <div className="flex flex-wrap gap-2">
                  {domains.slice(0, 10).map((domain: string) => (
                    <button
                      key={domain}
                      onClick={() => {
                        setQuery(`site:${domain}`);
                        setSearchTerm(`site:${domain}`);
                      }}
                      className="text-xs px-3 py-1 bg-gray-100 dark:bg-gray-700 rounded-full hover:bg-gray-200 dark:hover:bg-gray-600"
                    >
                      {domain}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <SearchInterface />
    </QueryClientProvider>
  );
}

export default App
