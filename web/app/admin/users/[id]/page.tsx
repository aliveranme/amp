"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Toaster, toast } from "sonner";
import { fetchRoutes, createRoute, deleteRoute, fetchUsers, updateUserName } from "@/lib/api";
import type { User, UserRoute } from "@/lib/types";

export default function UserDetailPage() {
  const { id } = useParams<{ id: string }>();
  const router = useRouter();
  const [routes, setRoutes] = useState<UserRoute[]>([]);
  const [user, setUser] = useState<User | null>(null);
  const [editingName, setEditingName] = useState(false);
  const [newName, setNewName] = useState("");
  const [open, setOpen] = useState(false);

  // Form state
  const [model, setModel] = useState("*");
  const [provider, setProvider] = useState("opencode");
  const [endpoint, setEndpoint] = useState("");
  const [apiKey, setApiKey] = useState("");

  const load = async () => {
    try {
      const [r, u] = await Promise.all([fetchRoutes(id), fetchUsers()]);
      setRoutes(r);
      setUser(u.find((x) => x.user_id === id) ?? null);
    } catch {
      toast.error("无法加载路由配置");
    }
  };
  useEffect(() => { load(); }, [id]);

  const handleRename = async () => {
    if (!newName.trim()) return;
    try {
      await updateUserName(id, newName);
      toast.success("名称已更新");
      setEditingName(false);
      await load();
    } catch {
      toast.error("更新失败");
    }
  };

  const handleAddRoute = async () => {
    if (!endpoint.trim()) return;
    try {
      await createRoute(id, { model, provider, endpoint, api_key: apiKey || undefined });
      toast.success("路由已添加");
      setOpen(false);
      setEndpoint("");
      setApiKey("");
      await load();
    } catch {
      toast.error("添加路由失败");
    }
  };

  const handleDeleteRoute = async (modelName: string) => {
    try {
      await deleteRoute(id, modelName);
      toast.success("路由已删除");
      await load();
    } catch {
      toast.error("删除失败");
    }
  };

  return (
    <div className="min-h-screen bg-background">
      <Toaster />
      <header className="border-b">
        <div className="container mx-auto px-4 py-4 flex items-center gap-4">
          <Button variant="ghost" onClick={() => router.push("/admin")}>
            ← 返回
          </Button>
          {editingName ? (
            <div className="flex gap-2 flex-1">
              <Input
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleRename()}
                placeholder="新名称"
              />
              <Button size="sm" onClick={handleRename}>保存</Button>
              <Button size="sm" variant="outline" onClick={() => setEditingName(false)}>取消</Button>
            </div>
          ) : (
            <>
              <h1 className="text-xl font-bold">{user?.name ?? "用户"}</h1>
              <Button size="sm" variant="outline" onClick={() => { setNewName(user?.name ?? ""); setEditingName(true); }}>
                编辑名称
              </Button>
            </>
          )}
          <Badge variant="outline" className="font-mono">
            {id.slice(0, 8)}...
          </Badge>
        </div>
      </header>

      <main className="container mx-auto px-4 py-8">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle>路由规则</CardTitle>
            <Dialog open={open} onOpenChange={setOpen}>
              <DialogTrigger render={<Button size="sm">添加路由</Button>} />
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>添加路由规则</DialogTitle>
                </DialogHeader>
                <div className="space-y-4">
                  <div>
                    <label className="text-sm font-medium">模型名</label>
                    <Select value={model} onValueChange={(v: string | null) => v && setModel(v)}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="*">*（所有模型）</SelectItem>
                        <SelectItem value="gpt-4o">gpt-4o</SelectItem>
                        <SelectItem value="gpt-4o-mini">gpt-4o-mini</SelectItem>
                        <SelectItem value="claude-sonnet-4">claude-sonnet-4</SelectItem>
                        <SelectItem value="claude-fable-5">claude-fable-5</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div>
                    <label className="text-sm font-medium">Provider</label>
                    <Input value={provider} onChange={(e) => setProvider(e.target.value)} />
                  </div>
                  <div>
                    <label className="text-sm font-medium">端点 URL</label>
                    <Input
                      placeholder="https://api.openai.com/v1/chat/completions"
                      value={endpoint}
                      onChange={(e) => setEndpoint(e.target.value)}
                    />
                  </div>
                  <div>
                    <label className="text-sm font-medium">Provider API Key</label>
                    <Input
                      type="password"
                      placeholder="sk-..."
                      value={apiKey}
                      onChange={(e) => setApiKey(e.target.value)}
                    />
                  </div>
                  <Button className="w-full" onClick={handleAddRoute}>
                    添加
                  </Button>
                </div>
              </DialogContent>
            </Dialog>
          </CardHeader>
          <CardContent>
            {routes.length === 0 ? (
              <p className="text-muted-foreground text-center py-8">
                还没有路由规则。点击 "添加路由" 配置。
              </p>
            ) : (
              <div className="space-y-3">
                {routes.map((r) => (
                  <div key={r.id} className="border rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <Badge>{r.model}</Badge>
                        <span className="text-sm text-muted-foreground">→</span>
                        <Badge variant="outline">{r.provider}</Badge>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="text-destructive"
                        onClick={() => handleDeleteRoute(r.model)}
                      >
                        删除
                      </Button>
                    </div>
                    <p className="text-xs text-muted-foreground font-mono truncate">
                      {r.endpoint}
                    </p>
                    <p className="text-xs text-muted-foreground mt-1">
                      Auth: {r.auth_header} | Key: {r.api_key_encrypted.slice(0, 8)}...
                    </p>
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
