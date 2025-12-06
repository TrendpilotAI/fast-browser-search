import { useState, useEffect } from 'react';
import { CommandPalette } from './ui/CommandPalette';
import { MainWindow } from './ui/MainWindow';

function App() {
  const [showCommandPalette, setShowCommandPalette] = useState(false);

  // Handle Cmd+K to toggle Command Palette
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setShowCommandPalette(prev => !prev);
      }
      if (e.key === 'Escape' && showCommandPalette) {
        setShowCommandPalette(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [showCommandPalette]);

  return (
    <div className="min-h-screen w-full bg-transparent font-sans text-text-primary antialiased selection:bg-accent-primary/30">
      <MainWindow />
      {showCommandPalette && (
        <CommandPalette onClose={() => setShowCommandPalette(false)} />
      )}
    </div>
  );
}

export default App;
