import type { Thread } from '@/lib/types';

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export async function fetchThreads(): Promise<Thread[]> {
  const res = await fetch(`${API_BASE}/api/threads`);
  return res.json();
}

export async function createThread(title: string): Promise<Thread> {
  const res = await fetch(`${API_BASE}/api/threads`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ title }),
  });
  return res.json();
}
