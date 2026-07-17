"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Toaster, toast } from "sonner";
import { fetchUsers, createUser, deleteUser, fetchStats } from "@/lib/api";
import type { User } from "@/lib/types";

export default function AdminPage() {
  const [users, setUsers] = useState<User[]>([]);
  const [stats, setStats] = useState({ user_count: 0, route_count: 0 });
  const [name, setName] = useState("");
  const [newKey, setNewKey] = useState("");
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const router = useRouter();

  const load = async () => {
    try {
      const [u, s] = await Promise.all([fetchUsers(), fetchStats()]);
      setUsers(u);
      setStats(s);
    } catch {
      toast.error("无法连接 BYOK 服务器");
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
      <header className="border-b">
        <div className="container mx-auto px-4 py-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <h1 className="text-xl font-bold">BYOK 管理</h1>
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
                  <pre className="bg-muted p-3 rounded text-sm break-all select-all">
                    {newKey}
                  </pre>
                  <Button
                    onClick={() => {
                      navigator.clipboard.writeText(newKey);
                      toast.success("已复制");
                    }}
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

      <main className="container mx-auto px-4 py-8 space-y-6">
        {/* Stats */}
        <div className="grid grid-cols-2 gap-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm text-muted-foreground">总用户</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-3xl font-bold">{stats.user_count}</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm text-muted-foreground">总路由</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-3xl font-bold">{stats.route_count}</p>
            </CardContent>
          </Card>
        </div>

        {/* Search */}
        <Input
          placeholder="搜索用户..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />

        {/* User List */}
        <Card>
          <CardHeader>
            <CardTitle>用户列表</CardTitle>
          </CardHeader>
          <CardContent>
            {filtered.length === 0 ? (
              <p className="text-muted-foreground text-center py-8">
                {search ? "无匹配用户" : "还没有用户。点击 创建用户 开始。"}
              </p>
            ) : (
              <div className="space-y-2">
                {filtered.map((u) => (
                  <div
                    key={u.user_id}
                    className="flex items-center justify-between p-3 rounded-lg border hover:bg-accent/50 cursor-pointer"
                    onClick={() => router.push(`/admin/users/${u.user_id}`)}
                  >
                    <div>
                      <p className="font-medium">{u.name}</p>
                      <p className="text-xs text-muted-foreground font-mono">
                        {u.user_id.slice(0, 8)}... | {u.created_at.slice(0, 10)}
                      </p>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-destructive"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(u.user_id);
                      }}
                    >
                      删除
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </main>
    </div>
  );
}
