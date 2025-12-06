import { describe, it, expect } from 'vitest';
import { generateRelevanceExplanation } from '../relevance';
import { SearchResult } from '../../lib/api';

describe('generateRelevanceExplanation', () => {
  it('should return explanation for high score', () => {
    const item: SearchResult = {
      id: '1',
      url: 'https://example.com',
      title: 'Test',
      source: 'chrome',
      score: 0.9,
    };
    const explanation = generateRelevanceExplanation(item, 'test');
    expect(explanation).toContain('Highly relevant');
  });

  it('should return explanation for title match', () => {
    const item: SearchResult = {
      id: '1',
      url: 'https://example.com',
      title: 'Hello World',
      source: 'chrome',
    };
    const explanation = generateRelevanceExplanation(item, 'hello');
    expect(explanation).toContain('Title matches');
  });

  it('should return explanation for topic match', () => {
    const item: SearchResult = {
      id: '1',
      url: 'https://example.com',
      title: 'Test',
      source: 'chrome',
      key_topics: ['react', 'typescript'],
    };
    const explanation = generateRelevanceExplanation(item, 'react');
    expect(explanation).toContain('Related to');
  });

  it('should return explanation for recent visit when no title match', () => {
    const item: SearchResult = {
      id: '1',
      url: 'https://example.com',
      title: 'Different Title',
      source: 'chrome',
      last_visit: new Date().toISOString(),
    };
    const explanation = generateRelevanceExplanation(item, 'xyz');
    expect(explanation).toContain('Recently visited');
  });

  it('should return explanation for high visit count when no title match', () => {
    const item: SearchResult = {
      id: '1',
      url: 'https://example.com',
      title: 'Different Title',
      source: 'chrome',
      visit_count: 15,
    };
    const explanation = generateRelevanceExplanation(item, 'xyz');
    expect(explanation).toContain('Frequently visited');
  });

  it('should return default explanation when no matches', () => {
    const item: SearchResult = {
      id: '1',
      url: 'https://example.com',
      title: 'Test',
      source: 'chrome',
    };
    const explanation = generateRelevanceExplanation(item, 'xyz');
    expect(explanation).toContain('Matches your search');
  });
});

