import type { Thread, User, UserRoute } from '@/lib/types';

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

// ─── Threads ─────────────────────────────────────────────────────

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

// ─── Admin: Users ────────────────────────────────────────────────

export async function fetchUsers(): Promise<User[]> {
  const res = await fetch(`${API_BASE}/admin/users`);
  const data = await res.json();
  return data.users ?? [];
}

export async function createUser(name: string): Promise<User> {
  const res = await fetch(`${API_BASE}/admin/users`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  });
  return res.json();
}

export async function deleteUser(userId: string): Promise<void> {
  await fetch(`${API_BASE}/admin/users/${userId}`, { method: 'DELETE' });
}

// ─── Admin: Routes ──────────────────────────────────────────────

export async function fetchRoutes(userId: string): Promise<UserRoute[]> {
  const res = await fetch(`${API_BASE}/admin/users/${userId}/routes`);
  return res.json();
}

export async function createRoute(
  userId: string,
  data: { model: string; provider: string; endpoint: string; api_key?: string }
): Promise<void> {
  await fetch(`${API_BASE}/admin/users/${userId}/routes`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function deleteRoute(userId: string, model: string): Promise<void> {
  await fetch(`${API_BASE}/admin/users/${userId}/routes/${model}`, {
    method: 'DELETE',
  });
}
