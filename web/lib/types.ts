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
