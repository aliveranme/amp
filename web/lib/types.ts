export interface Thread {
  id: string;
  title: string;
  status: 'Active' | 'Archived';
  messages: Message[];
  created_at: string;
  updated_at: string;
}

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  timestamp: string;
}

export interface Session {
  id: string;
  thread_id: string;
  agent_mode: 'low' | 'medium' | 'high' | 'ultra';
  status: 'Active' | 'Paused' | 'Ended';
  started_at: string;
  last_heartbeat: string;
}

// ─── Admin: User Management ──────────────────────────────────────

export interface User {
  api_key: string;
  user_id: string;
  name: string;
  created_at: string;
}

export interface UserRoute {
  id: number;
  user_id: string;
  model: string;
  provider: string;
  endpoint: string;
  auth_header: string;
  api_key_encrypted: string;
  created_at: string;
}
