/**
 * Master-Class Command Palette App
 * Replaces the old search interface with the new CommandPalette
 */

import { useState, useEffect } from 'react';
import { CommandPalette } from './CommandPalette';
import { Sparkles } from 'lucide-react';

function App() {
  const [isOpen, setIsOpen] = useState(true); // Start open for demo, can be toggled with Cmd+K

  // Keyboard shortcut: Cmd+K or Ctrl+K to toggle
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsOpen((prev) => !prev);
      }
      if (e.key === 'Escape' && isOpen) {
        setIsOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen]);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Fallback UI when palette is closed */}
      {!isOpen && (
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-center">
            <Sparkles className="w-16 h-16 text-blue-500 mx-auto mb-4" />
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
              Fast Browser Search
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              Press <kbd className="px-2 py-1 bg-gray-200 dark:bg-gray-700 rounded text-sm">⌘K</kbd> or <kbd className="px-2 py-1 bg-gray-200 dark:bg-gray-700 rounded text-sm">Ctrl+K</kbd> to open the command palette
            </p>
            <button
              onClick={() => setIsOpen(true)}
              className="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
            >
              Open Command Palette
            </button>
          </div>
        </div>
      )}

      {/* Command Palette */}
      {isOpen && <CommandPalette onClose={() => setIsOpen(false)} />}
    </div>
  );
}

export default App;
