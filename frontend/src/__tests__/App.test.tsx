import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import App from '../App';

// Mock MainWindow and CommandPalette
vi.mock('../ui/MainWindow', () => ({
  MainWindow: () => <div data-testid="main-window">MainWindow</div>,
}));

vi.mock('../ui/CommandPalette', () => ({
  CommandPalette: ({ onClose }: { onClose?: () => void }) => (
    <div data-testid="command-palette">
      CommandPalette
      {onClose && <button onClick={onClose}>Close</button>}
    </div>
  ),
}));

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render MainWindow by default', () => {
    render(<App />);
    expect(screen.getByTestId('main-window')).toBeInTheDocument();
  });

  it('should not render CommandPalette by default', () => {
    render(<App />);
    expect(screen.queryByTestId('command-palette')).not.toBeInTheDocument();
  });

  it('should open CommandPalette on Cmd+K', async () => {
    render(<App />);
    
    // Simulate Cmd+K
    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    
    await waitFor(() => {
      expect(screen.getByTestId('command-palette')).toBeInTheDocument();
    });
  });

  it('should close CommandPalette on Escape', async () => {
    render(<App />);
    
    // Open CommandPalette
    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    
    await waitFor(() => {
      expect(screen.getByTestId('command-palette')).toBeInTheDocument();
    });
    
    // Close CommandPalette
    fireEvent.keyDown(window, { key: 'Escape' });
    
    await waitFor(() => {
      expect(screen.queryByTestId('command-palette')).not.toBeInTheDocument();
    });
  });
});


