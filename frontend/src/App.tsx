import { CommandPalette } from './ui/CommandPalette';

function App() {
  return (
    <div className="min-h-screen w-full bg-transparent font-sans text-text-primary antialiased selection:bg-accent-primary/30">
      <CommandPalette />
    </div>
  );
}

export default App;
