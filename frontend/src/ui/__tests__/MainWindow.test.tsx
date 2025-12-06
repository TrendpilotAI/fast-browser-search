import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MainWindow } from '../MainWindow';
import * as tauri from '../../lib/tauri';

// Mock Tauri API
vi.mock('../../lib/tauri', () => ({
  connectGoogle: vi.fn(),
  reIndex: vi.fn(),
  getGmailStatus: vi.fn(() => Promise.resolve(false)),
}));

// Mock useMainWindowSearch hook
vi.mock('../../hooks/useMainWindowSearch', () => ({
  useMainWindowSearch: () => ({
    query: '',
    setQuery: vi.fn(),
    results: [],
    suggestions: [],
    isLoading: false,
    selectedIndex: 0,
    setSelectedIndex: vi.fn(),
    handleNavigation: vi.fn(),
    addToHistory: vi.fn(),
    browserFilters: {
      chrome: false,
      safari: false,
      arc: false,
      comet: false,
      genspark: false,
      thorium: false,
      gmail: false,
    },
    toggleBrowserFilter: vi.fn(),
  }),
}));

describe('MainWindow', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render MainWindow component', () => {
    render(<MainWindow />);
    expect(screen.getByText('Ultra Fast Search')).toBeInTheDocument();
  });

  it('should render Connect Gmail button', () => {
    render(<MainWindow />);
    expect(screen.getByText('Connect Gmail')).toBeInTheDocument();
  });

  it('should render Re-index button', () => {
    render(<MainWindow />);
    expect(screen.getByText('Re-index')).toBeInTheDocument();
  });

  it('should render search input', () => {
    render(<MainWindow />);
    expect(screen.getByPlaceholderText(/Search here or press/)).toBeInTheDocument();
  });

  it('should render browser filter chips', () => {
    render(<MainWindow />);
    expect(screen.getByText('Chrome')).toBeInTheDocument();
    expect(screen.getByText('Safari')).toBeInTheDocument();
    expect(screen.getByText('Arc')).toBeInTheDocument();
  });

  it('should call connectGoogle when Connect Gmail button is clicked', async () => {
    vi.mocked(tauri.connectGoogle).mockResolvedValue('Success');
    render(<MainWindow />);
    
    const button = screen.getByText('Connect Gmail');
    fireEvent.click(button);
    
    await waitFor(() => {
      expect(tauri.connectGoogle).toHaveBeenCalled();
    });
  });

  it('should call reIndex when Re-index button is clicked', async () => {
    vi.mocked(tauri.reIndex).mockResolvedValue('Success');
    render(<MainWindow />);
    
    const button = screen.getByText('Re-index');
    fireEvent.click(button);
    
    await waitFor(() => {
      expect(tauri.reIndex).toHaveBeenCalled();
    });
  });
});


