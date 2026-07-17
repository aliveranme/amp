import type { Thread, User, UserRoute } from '@/lib/types';

async function apiFetch<T>(url: string, opts?: RequestInit): Promise<T> {
  const res = await fetch(url, opts);
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`HTTP ${res.status}: ${body.slice(0, 200)}`);
  }
  return res.json();
}

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

// ─── Stats ────────────────────────────────────────────────────────

export async function fetchStats(): Promise<{ user_count: number; route_count: number }> {
  return apiFetch(`${API_BASE}/admin/api/stats`);
}

// ─── Threads ─────────────────────────────────────────────────────

export async function fetchThreads(): Promise<Thread[]> {
  return apiFetch(`${API_BASE}/api/threads`);
}

export async function createThread(title: string): Promise<Thread> {
  return apiFetch(`${API_BASE}/api/threads`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ title }),
  });
}

// ─── Admin: Users ────────────────────────────────────────────────

export async function fetchUsers(): Promise<User[]> {
  const data = await apiFetch<{ users: User[] }>(`${API_BASE}/admin/api/users`);
  return data.users ?? [];
}

export async function fetchUser(userId: string): Promise<User> {
  return apiFetch<User>(`${API_BASE}/admin/api/users/${userId}`);
}

export async function createUser(name: string): Promise<User> {
  return apiFetch<User>(`${API_BASE}/admin/api/users`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  });
}

export async function deleteUser(userId: string): Promise<void> {
  await apiFetch(`${API_BASE}/admin/api/users/${userId}`, { method: 'DELETE' });
}

export async function updateUserName(userId: string, name: string): Promise<void> {
  await apiFetch(`${API_BASE}/admin/api/users/${userId}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  });
}

// ─── Admin: Routes ──────────────────────────────────────────────

export async function fetchRoutes(userId: string): Promise<UserRoute[]> {
  return apiFetch(`${API_BASE}/admin/api/users/${userId}/routes`);
}

export async function createRoute(
  userId: string,
  data: { model: string; provider: string; endpoint: string; api_key?: string }
): Promise<void> {
  await apiFetch(`${API_BASE}/admin/api/users/${userId}/routes`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
}

export async function deleteRoute(userId: string, model: string): Promise<void> {
  await apiFetch(`${API_BASE}/admin/api/users/${userId}/routes/${model}`, {
    method: 'DELETE',
  });
}
