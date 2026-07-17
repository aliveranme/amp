"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
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
import { fetchRoutes, createRoute, deleteRoute, fetchUser, updateUserName, toggleRoute, fetchUserUsage } from "@/lib/api";
import type { User, UserRoute } from "@/lib/types";

function RouteCard({ route, onDelete, onToggle }: { route: UserRoute; onDelete: () => void; onToggle: () => void }) {
  return (
    <div className="flex items-center justify-between p-4 border rounded-lg hover:bg-accent/30 transition-colors">
      <div className="space-y-1 min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <Badge className={`font-mono ${!route.enabled ? 'opacity-50' : ''}`}>{route.model}</Badge>
          <span className="text-xs text-muted-foreground">→</span>
          <Badge variant={route.enabled ? "secondary" : "outline"}>{route.provider}</Badge>
          {!route.enabled && <Badge variant="outline" className="text-xs">已停用</Badge>}
        </div>
        <p className="text-xs text-muted-foreground font-mono truncate">{route.endpoint}</p>
        <p className="text-xs text-muted-foreground">
          认证: <span className="font-mono">{route.auth_header}</span>
          {route.api_key_encrypted && <> · Key: <span className="font-mono">{route.api_key_encrypted.slice(0, 8)}…</span></>}
          {route.rate_limit > 0 && <> · 限流: {route.rate_limit}/分</>}
        </p>
      </div>
      <div className="flex items-center gap-2 shrink-0 ml-4">
        <Button variant="ghost" size="sm" onClick={onToggle}>
          {route.enabled ? "停用" : "启用"}
        </Button>
        <Button variant="ghost" size="sm" className="text-destructive" onClick={onDelete}>
          删除
        </Button>
      </div>
    </div>
  );
}

export default function UserDetailPage() {
  const { id } = useParams<{ id: string }>();
  const router = useRouter();
  const [routes, setRoutes] = useState<UserRoute[]>([]);
  const [user, setUser] = useState<User | null>(null);
  const [usage, setUsage] = useState({ total_requests: 0, total_tokens_in: 0, total_tokens_out: 0, by_model: [] as Array<{ model: string; requests: number; tokens_in: number; tokens_out: number }> });
  const [loading, setLoading] = useState(true);
  const [editingName, setEditingName] = useState(false);
  const [newName, setNewName] = useState("");
  const [open, setOpen] = useState(false);

  // Route form state
  const [model, setModel] = useState("*");
  const [provider, setProvider] = useState("opencode");
  const [endpoint, setEndpoint] = useState("");
  const [apiKey, setApiKey] = useState("");

  const load = async () => {
    setLoading(true);
    try {
      const [r, u, usg] = await Promise.all([fetchRoutes(id), fetchUser(id), fetchUserUsage(id)]);
      setRoutes(r);
      setUser(u);
      setUsage(usg);
    } catch (e) {
      toast.error("加载失败: " + (e instanceof Error ? e.message : "未知错误"));
    } finally {
      setLoading(false);
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
    } catch (e) {
      toast.error("更新失败: " + (e instanceof Error ? e.message : ""));
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

  const handleToggleRoute = async (modelName: string, enabled: boolean) => {
    try {
      await toggleRoute(id, modelName, !enabled);
      toast.success(enabled ? "已停用" : "已启用");
      await load();
    } catch {
      toast.error("操作失败");
    }
  };

  return (
    <div className="min-h-screen bg-background">
      <Toaster />

      {/* Header */}
      <header className="border-b bg-card">
        <div className="container mx-auto px-4 py-3 flex items-center gap-3">
          <Button variant="ghost" size="sm" onClick={() => router.push("/admin")}>
            ← 返回
          </Button>
          {editingName ? (
            <div className="flex items-center gap-2 flex-1 max-w-md">
              <Input
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleRename()}
                placeholder="新名称"
                autoFocus
              />
              <Button size="sm" onClick={handleRename}>保存</Button>
              <Button size="sm" variant="outline" onClick={() => setEditingName(false)}>取消</Button>
            </div>
          ) : (
            <div className="flex items-center gap-2 flex-1">
              {loading ? (
                <Skeleton className="h-6 w-40" />
              ) : (
                <>
                  <Avatar className="h-7 w-7">
                    <AvatarFallback className="text-[10px]">{(user?.name ?? "?").slice(0, 2).toUpperCase()}</AvatarFallback>
                  </Avatar>
                  <h1 className="text-lg font-semibold">{user?.name ?? "用户"}</h1>
                  <Button size="sm" variant="outline" disabled={loading} onClick={() => { setNewName(user?.name ?? ""); setEditingName(true); }}>
                    编辑名称
                  </Button>
                </>
              )}
            </div>
          )}
          <Badge variant="outline" className="font-mono text-xs">{id.slice(0, 8)}…</Badge>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6 space-y-6">
        {/* User Info Section */}
        <Card>
          <CardHeader><CardTitle className="text-base">用户信息</CardTitle></CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-2">
                <Skeleton className="h-4 w-48" />
                <Skeleton className="h-4 w-64" />
                <Skeleton className="h-4 w-32" />
              </div>
            ) : user ? (
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                <div>
                  <p className="text-muted-foreground text-xs">API Key</p>
                  <p className="font-mono text-xs mt-0.5 break-all">{user.api_key}</p>
                </div>
                <div>
                  <p className="text-muted-foreground text-xs">用户 ID</p>
                  <p className="font-mono text-xs mt-0.5">{user.user_id}</p>
                </div>
                <div>
                  <p className="text-muted-foreground text-xs">创建时间</p>
                  <p className="text-xs mt-0.5">{user.created_at.slice(0, 19).replace("T", " ")}</p>
                </div>
              </div>
            ) : null}
          </CardContent>
        </Card>

        {/* Routes Section */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-base">路由规则</CardTitle>
            <Dialog open={open} onOpenChange={setOpen}>
              <DialogTrigger render={<Button size="sm">添加路由</Button>} />
              <DialogContent>
                <DialogHeader><DialogTitle>添加路由规则</DialogTitle></DialogHeader>
                <div className="space-y-4">
                  <div>
                    <label className="text-sm font-medium">模型名</label>
                    <Select value={model} onValueChange={(v) => v && setModel(v)}>
                      <SelectTrigger><SelectValue /></SelectTrigger>
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
                    <Input placeholder="https://api.openai.com/v1/chat/completions" value={endpoint} onChange={(e) => setEndpoint(e.target.value)} />
                  </div>
                  <div>
                    <label className="text-sm font-medium">Provider API Key</label>
                    <Input type="password" placeholder="sk-..." value={apiKey} onChange={(e) => setApiKey(e.target.value)} />
                  </div>
                  <Button className="w-full" onClick={handleAddRoute}>添加</Button>
                </div>
              </DialogContent>
            </Dialog>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-3">
                {[...Array(2)].map((_, i) => <Skeleton key={i} className="h-20 w-full rounded-lg" />)}
              </div>
            ) : routes.length === 0 ? (
              <p className="text-muted-foreground text-center py-8 text-sm">还没有路由规则。点击"添加路由"配置。</p>
            ) : (
              <div className="space-y-3">
                {routes.map((r) => (
                  <RouteCard key={r.id} route={r} onDelete={() => handleDeleteRoute(r.model)} onToggle={() => handleToggleRoute(r.model, r.enabled)} />
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Usage Section */}
        <Card>
          <CardHeader><CardTitle className="text-base">用量统计 (7天)</CardTitle></CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-2"><Skeleton className="h-4 w-40" /><Skeleton className="h-4 w-32" /></div>
            ) : usage.total_requests === 0 ? (
              <p className="text-muted-foreground text-sm text-center py-4">暂无用量数据</p>
            ) : (
              <div className="space-y-4">
                <div className="grid grid-cols-3 gap-4 text-center">
                  <div><p className="text-2xl font-bold">{usage.total_requests}</p><p className="text-xs text-muted-foreground">请求数</p></div>
                  <div><p className="text-2xl font-bold">{(usage.total_tokens_in / 1000).toFixed(1)}K</p><p className="text-xs text-muted-foreground">输入 Token</p></div>
                  <div><p className="text-2xl font-bold">{(usage.total_tokens_out / 1000).toFixed(1)}K</p><p className="text-xs text-muted-foreground">输出 Token</p></div>
                </div>
                {usage.by_model.length > 0 && (
                  <>
                    <Separator />
                    <p className="text-sm font-medium">按模型</p>
                    <div className="space-y-2">
                      {usage.by_model.map((m) => (
                        <div key={m.model} className="flex items-center justify-between text-sm">
                          <Badge variant="outline" className="font-mono text-xs">{m.model}</Badge>
                          <span className="text-xs text-muted-foreground">{m.requests} 请求 · {m.tokens_in + m.tokens_out} tokens</span>
                        </div>
                      ))}
                    </div>
                  </>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      </main>
    </div>
  );
}
