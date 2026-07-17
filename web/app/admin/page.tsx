"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Skeleton } from "@/components/ui/skeleton";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Toaster, toast } from "sonner";
import { fetchUsers, createUser, deleteUser, fetchStats, fetchThreads } from "@/lib/api";
import type { User, Thread } from "@/lib/types";

function UserAvatar({ name }: { name: string }) {
  return (
    <Avatar className="h-8 w-8">
      <AvatarFallback className="text-xs">{name.slice(0, 2).toUpperCase()}</AvatarFallback>
    </Avatar>
  );
}

export default function AdminPage() {
  const [users, setUsers] = useState<User[]>([]);
  const [threads, setThreads] = useState<Thread[]>([]);
  const [stats, setStats] = useState({ user_count: 0, route_count: 0, usage_7d: { total_requests: 0, total_tokens_in: 0, total_tokens_out: 0 } });
  const [loading, setLoading] = useState(true);
  const [name, setName] = useState("");
  const [newKey, setNewKey] = useState("");
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const router = useRouter();

  const load = async () => {
    setLoading(true);
    try {
      const [u, s, t] = await Promise.all([fetchUsers(), fetchStats(), fetchThreads()]);
      setUsers(u);
      setStats(s);
      setThreads(t);
    } catch {
      toast.error("无法连接 BYOK 服务器");
    } finally {
      setLoading(false);
    }
  };
  useEffect(() => { load(); }, []);

  const handleCreate = async () => {
    if (!name.trim()) return;
    try {
      const user = await createUser(name);
      setNewKey(user.api_key);
      setName("");
      await load();
    } catch {
      toast.error("创建失败");
    }
  };

  const handleDelete = async (userId: string) => {
    if (!confirm("确定删除此用户？")) return;
    try {
      await deleteUser(userId);
      toast.success("已删除");
      await load();
    } catch {
      toast.error("删除失败");
    }
  };

  const filtered = users.filter(
    (u) =>
      u.name.toLowerCase().includes(search.toLowerCase()) ||
      u.user_id.includes(search)
  );

  return (
    <div className="min-h-screen bg-background">
      <Toaster />
      <header className="border-b bg-card">
        <div className="container mx-auto px-4 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <h1 className="text-xl font-bold tracking-tight">BYOK 管理</h1>
            <Badge variant="secondary">{stats.user_count} 用户</Badge>
            <Badge variant="outline">{stats.route_count} 路由</Badge>
          </div>
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger render={<Button>创建用户</Button>} />
            <DialogContent>
              <DialogHeader>
                <DialogTitle>{newKey ? "用户创建成功" : "创建新用户"}</DialogTitle>
              </DialogHeader>
              {newKey ? (
                <div className="space-y-4">
                  <p className="text-sm text-muted-foreground">
                    请保存以下 API Key——关闭后不再显示：
                  </p>
                  <pre className="bg-muted p-3 rounded text-sm break-all select-all font-mono">
                    {newKey}
                  </pre>
                  <Button
                    onClick={() => { navigator.clipboard.writeText(newKey); toast.success("已复制"); }}
                    className="w-full"
                  >
                    复制 API Key
                  </Button>
                  <Button
                    variant="outline"
                    className="w-full"
                    onClick={() => { setNewKey(""); setOpen(false); }}
                  >
                    完成
                  </Button>
                </div>
              ) : (
                <div className="flex gap-2">
                  <Input
                    placeholder="用户名"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleCreate()}
                  />
                  <Button onClick={handleCreate}>创建</Button>
                </div>
              )}
            </DialogContent>
          </Dialog>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6">
        <Tabs defaultValue="users">
          <TabsList className="mb-6">
            <TabsTrigger value="users">用户管理</TabsTrigger>
            <TabsTrigger value="overview">总览</TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="space-y-6">
            {/* Stats Grid */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {loading ? (
                <>
                  {[...Array(4)].map((_, i) => (
                    <Card key={i}><CardContent className="pt-6"><Skeleton className="h-4 w-20 mb-2" /><Skeleton className="h-8 w-16" /></CardContent></Card>
                  ))}
                </>
              ) : (
                <>
                  <Card><CardContent className="pt-6"><p className="text-sm text-muted-foreground">用户数</p><p className="text-3xl font-bold mt-1">{stats.user_count}</p></CardContent></Card>
                  <Card><CardContent className="pt-6"><p className="text-sm text-muted-foreground">路由数</p><p className="text-3xl font-bold mt-1">{stats.route_count}</p></CardContent></Card>
                  <Card><CardContent className="pt-6"><p className="text-sm text-muted-foreground">请求数(7d)</p><p className="text-3xl font-bold mt-1">{stats.usage_7d.total_requests}</p></CardContent></Card>
                  <Card><CardContent className="pt-6"><p className="text-sm text-muted-foreground">Token(7d)</p><p className="text-3xl font-bold mt-1">{stats.usage_7d.total_tokens_in + stats.usage_7d.total_tokens_out}</p></CardContent></Card>
                </>
              )}
            </div>

            {/* Recent Threads */}
            <Card>
              <CardHeader><CardTitle>最近线程</CardTitle></CardHeader>
              <CardContent>
                {loading ? (
                  <div className="space-y-3">{[...Array(3)].map((_, i) => <Skeleton key={i} className="h-12 w-full" />)}</div>
                ) : threads.length === 0 ? (
                  <p className="text-muted-foreground text-center py-6 text-sm">暂无线程</p>
                ) : (
                  <div className="divide-y">
                    {threads.slice(0, 5).map((t) => (
                      <div key={t.id} className="py-3 flex items-center justify-between">
                        <div>
                          <p className="text-sm font-medium">{t.title || "未命名线程"}</p>
                          <p className="text-xs text-muted-foreground font-mono">{t.id.slice(0, 8)}...</p>
                        </div>
                        <Badge variant={t.status === "Active" ? "default" : "secondary"}>{t.status}</Badge>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="users" className="space-y-4">
            {/* Search */}
            <div className="flex gap-2">
              <Input
                placeholder="搜索用户名或 ID..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                className="max-w-sm"
              />
              {search && (
                <Button variant="ghost" size="sm" onClick={() => setSearch("")}>清除</Button>
              )}
            </div>

            {/* User List */}
            <Card>
              <CardContent className="p-0">
                {loading ? (
                  <div className="p-4 space-y-4">
                    {[...Array(3)].map((_, i) => (
                      <div key={i} className="flex items-center gap-3">
                        <Skeleton className="h-8 w-8 rounded-full" />
                        <div className="flex-1"><Skeleton className="h-4 w-32 mb-1" /><Skeleton className="h-3 w-20" /></div>
                        <Skeleton className="h-8 w-16" />
                      </div>
                    ))}
                  </div>
                ) : filtered.length === 0 ? (
                  <p className="text-muted-foreground text-center py-12 text-sm">
                    {search ? `未找到 "${search}"` : "还没有用户。点击右上角创建用户。"}
                  </p>
                ) : (
                  <div className="divide-y">
                    {filtered.map((u) => (
                      <div
                        key={u.user_id}
                        className="flex items-center gap-3 px-4 py-3 hover:bg-accent/50 cursor-pointer transition-colors"
                        onClick={() => router.push(`/admin/users/${u.user_id}`)}
                      >
                        <UserAvatar name={u.name} />
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium truncate">{u.name}</p>
                          <p className="text-xs text-muted-foreground font-mono truncate">
                            {u.user_id.slice(0, 8)}... · {u.created_at.slice(0, 10)}
                          </p>
                        </div>
                        <Badge variant="outline" className="text-xs font-mono">
                          {u.api_key.slice(0, 12)}...
                        </Badge>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-destructive shrink-0"
                          onClick={(e) => { e.stopPropagation(); handleDelete(u.user_id); }}
                        >
                          删除
                        </Button>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>

            {!loading && filtered.length > 0 && (
              <p className="text-xs text-muted-foreground text-center">
                共 {filtered.length} 个用户{search !== "" && `（匹配 "${search}"）`}
              </p>
            )}
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}
