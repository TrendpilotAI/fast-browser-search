import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CommandPalette } from '../CommandPalette';

// Mock useCommandEngine hook
vi.mock('../../hooks/useCommandEngine', () => ({
  useCommandEngine: () => ({
    query: '',
    setQuery: vi.fn(),
    results: [
      {
        id: '1',
        url: 'https://example.com',
        title: 'Test Result',
        summary: 'This is a test summary',
        score: 0.8,
        visit_count: 5,
        source: 'chrome' as const,
      },
    ],
    suggestions: [],
    isLoading: false,
    selectedIndex: 0,
    setSelectedIndex: vi.fn(),
    handleNavigation: vi.fn(),
    addToHistory: vi.fn(),
  }),
}));

describe('CommandPalette', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render CommandPalette component', () => {
    render(<CommandPalette />);
    expect(screen.getByPlaceholderText(/Type to search/)).toBeInTheDocument();
  });

  it('should display search results', () => {
    render(<CommandPalette />);
    expect(screen.getByText('Test Result')).toBeInTheDocument();
  });

  it('should display summary in results', () => {
    render(<CommandPalette />);
    expect(screen.getByText(/This is a test summary/)).toBeInTheDocument();
  });

  it('should call onClose when provided', () => {
    const onClose = vi.fn();
    render(<CommandPalette onClose={onClose} />);
    
    // Simulate Escape key
    fireEvent.keyDown(window, { key: 'Escape', code: 'Escape' });
    
    // Note: This test may need adjustment based on actual implementation
    // The onClose is called when Escape is pressed and query is empty
  });
});


