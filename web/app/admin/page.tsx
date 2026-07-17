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
import { fetchUsers, createUser, deleteUser } from "@/lib/api";
import type { User } from "@/lib/types";

export default function AdminPage() {
  const [users, setUsers] = useState<User[]>([]);
  const [name, setName] = useState("");
  const [newKey, setNewKey] = useState("");
  const [open, setOpen] = useState(false);
  const router = useRouter();

  const load = async () => {
    try {
      setUsers(await fetchUsers());
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

  return (
    <div className="min-h-screen bg-background">
      <Toaster />
      <header className="border-b">
        <div className="container mx-auto px-4 py-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <h1 className="text-xl font-bold">BYOK 管理</h1>
            <Badge variant={users.length > 0 ? "default" : "secondary"}>
              {users.length} 用户
            </Badge>
          </div>
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger>
              <Button>创建用户</Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>
                  {newKey ? "用户创建成功" : "创建新用户"}
                </DialogTitle>
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
                    onClick={() => {
                      setNewKey("");
                      setOpen(false);
                    }}
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

      <main className="container mx-auto px-4 py-8">
        <Card>
          <CardHeader>
            <CardTitle>用户列表</CardTitle>
          </CardHeader>
          <CardContent>
            {users.length === 0 ? (
              <p className="text-muted-foreground text-center py-8">
                还没有用户。点击 "创建用户" 开始。
              </p>
            ) : (
              <div className="space-y-2">
                {users.map((u) => (
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
