import { invoke } from '@tauri-apps/api/tauri';

/**
 * Tauri API wrapper for app-specific commands
 */

export interface GmailStatus {
  connected: boolean;
}

/**
 * Connect to Google/Gmail via OAuth
 * Opens browser for OAuth flow
 */
export async function connectGoogle(): Promise<string> {
  try {
    const result = await invoke<string>('connect_google');
    return result;
  } catch (error) {
    throw new Error(`Failed to connect Google: ${error}`);
  }
}

/**
 * Re-index all browser histories
 * Returns immediately, indexing happens in background
 */
export async function reIndex(): Promise<string> {
  try {
    const result = await invoke<string>('index_history');
    return result;
  } catch (error) {
    throw new Error(`Failed to re-index: ${error}`);
  }
}

/**
 * Get Gmail connection status
 */
export async function getGmailStatus(): Promise<boolean> {
  try {
    const result = await invoke<boolean>('gmail_status');
    return result;
  } catch (error) {
    console.error('Failed to get Gmail status:', error);
    return false;
  }
}

