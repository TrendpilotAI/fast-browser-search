import { SearchResult } from '../lib/api';

/**
 * Generate a one-sentence explanation of why a search result is relevant
 */
export function generateRelevanceExplanation(item: SearchResult, query: string): string {
  const explanations: string[] = [];
  const queryLower = query.toLowerCase();
  
  // High semantic score (highest priority)
  if (item.score !== undefined && item.score > 0.8) {
    explanations.push('Highly relevant to your search');
  } else if (item.score !== undefined && item.score > 0.6) {
    explanations.push('Relevant to your search');
  }
  
  // Title match (high priority)
  const titleLower = (item.title || '').toLowerCase();
  if (titleLower.includes(queryLower)) {
    explanations.push('Title matches your query');
  }
  
  // Topic match
  if (item.key_topics && item.key_topics.length > 0) {
    const matchingTopics = item.key_topics.filter(topic => 
      queryLower.includes(topic.toLowerCase()) || 
      topic.toLowerCase().includes(queryLower)
    );
    if (matchingTopics.length > 0) {
      explanations.push(`Related to ${matchingTopics.slice(0, 2).join(' and ')}`);
    }
  }
  
  // Recent visit (only if no title match to avoid redundancy)
  if (item.last_visit && !titleLower.includes(queryLower)) {
    const date = new Date(item.last_visit);
    const diff = Date.now() - date.getTime();
    if (diff < 7 * 24 * 60 * 60 * 1000) { // Within last week
      explanations.push('Recently visited');
    }
  }
  
  // High visit count (only if no title match)
  if (item.visit_count && item.visit_count > 10 && !titleLower.includes(queryLower)) {
    explanations.push('Frequently visited');
  }
  
  // Site category match
  if (item.site_category) {
    const categoryLower = item.site_category.toLowerCase();
    if (queryLower.includes(categoryLower) || categoryLower.includes(queryLower)) {
      explanations.push(`From ${item.site_category} category`);
    }
  }
  
  // Return first explanation or default
  if (explanations.length > 0) {
    return explanations[0];
  }
  
  // Default fallback
  return 'Matches your search criteria';
}

