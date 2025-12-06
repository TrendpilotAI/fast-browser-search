import { describe, it, expect } from 'vitest';
import { highlightSearchTerms, highlightSearchTermsString } from '../highlight';
import { render } from '@testing-library/react';
import React from 'react';

describe('highlightSearchTerms', () => {
  it('should highlight matching words case-insensitively', () => {
    const result = render(React.createElement(React.Fragment, null, highlightSearchTerms('Hello World', 'hello')));
    expect(result.container.innerHTML).toContain('<mark');
  });

  it('should handle empty query', () => {
    const result = render(React.createElement(React.Fragment, null, highlightSearchTerms('Hello World', '')));
    expect(result.container.textContent).toBe('Hello World');
  });

  it('should handle empty text', () => {
    const result = render(React.createElement(React.Fragment, null, highlightSearchTerms('', 'hello')));
    expect(result.container.textContent).toBe('');
  });

  it('should highlight multiple words', () => {
    const result = render(React.createElement(React.Fragment, null, highlightSearchTerms('Hello World Test', 'hello test')));
    const html = result.container.innerHTML;
    expect(html).toContain('<mark');
  });

  it('should handle special characters in query', () => {
    const result = render(React.createElement(React.Fragment, null, highlightSearchTerms('Hello (World)', '(world')));
    expect(result.container.innerHTML).toContain('<mark');
  });

  it('should preserve original text structure', () => {
    const text = 'Hello World';
    const result = render(React.createElement(React.Fragment, null, highlightSearchTerms(text, 'world')));
    expect(result.container.textContent).toBe(text);
  });
});

describe('highlightSearchTermsString', () => {
  it('should return string with mark tags', () => {
    const result = highlightSearchTermsString('Hello World', 'hello');
    expect(result).toContain('<mark>');
    expect(result).toContain('Hello');
  });

  it('should handle empty query', () => {
    const result = highlightSearchTermsString('Hello World', '');
    expect(result).toBe('Hello World');
  });
});

