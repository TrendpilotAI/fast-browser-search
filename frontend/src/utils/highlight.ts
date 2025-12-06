import React from 'react';

/**
 * Highlight search terms in text
 */

/**
 * Highlights matching search terms in text
 * @param text - The text to highlight
 * @param query - The search query
 * @returns JSX element with highlighted terms
 */
export function highlightSearchTerms(text: string, query: string): React.ReactNode {
  if (!query || !text) {
    return text;
  }

  // Split query into words, filtering out empty strings
  const queryWords = query
    .toLowerCase()
    .split(/\s+/)
    .filter(word => word.length > 0);

  if (queryWords.length === 0) {
    return text;
  }

  // Create regex pattern that matches any of the query words (case-insensitive)
  const pattern = new RegExp(`(${queryWords.map(word => escapeRegex(word)).join('|')})`, 'gi');
  
  // Split text by matches while preserving the matches
  const parts: (string | React.ReactElement)[] = [];
  let lastIndex = 0;
  let match;
  let keyCounter = 0;

  // Reset regex lastIndex
  pattern.lastIndex = 0;
  
  while ((match = pattern.exec(text)) !== null) {
    // Add text before match
    if (match.index > lastIndex) {
      parts.push(text.substring(lastIndex, match.index));
    }
    
    // Add highlighted match
    parts.push(
      React.createElement(
        'mark',
        { 
          key: `highlight-${keyCounter++}`,
          className: "bg-accent-primary/30 text-accent-primary font-medium px-0.5 rounded"
        },
        match[0]
      )
    );
    
    lastIndex = pattern.lastIndex;
  }
  
  // Add remaining text
  if (lastIndex < text.length) {
    parts.push(text.substring(lastIndex));
  }
  
  // If no matches found, return original text
  if (parts.length === 0) {
    return text;
  }
  
  return React.createElement(React.Fragment, null, ...parts);
}

/**
 * Escape special regex characters
 */
function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Simple string version (for non-React contexts)
 */
export function highlightSearchTermsString(text: string, query: string): string {
  if (!query || !text) {
    return text;
  }

  const queryWords = query
    .toLowerCase()
    .split(/\s+/)
    .filter(word => word.length > 0);

  if (queryWords.length === 0) {
    return text;
  }

  const pattern = new RegExp(`(${queryWords.map(word => escapeRegex(word)).join('|')})`, 'gi');
  return text.replace(pattern, '<mark>$1</mark>');
}

