"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { fetchThreads, createThread } from "@/lib/api";
import type { Thread } from "@/lib/types";

export default function Dashboard() {
  const [threads, setThreads] = useState<Thread[]>([]);
  const [title, setTitle] = useState("");

  useEffect(() => {
    fetchThreads().then(setThreads).catch(console.error);
  }, []);

  const handleCreate = async () => {
    if (!title.trim()) return;
    const newThread = await createThread(title);
    setThreads([newThread, ...threads]);
    setTitle("");
  };

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b">
        <div className="container mx-auto px-4 py-4 flex items-center justify-between">
          <h1 className="text-xl font-bold">amp code BYOK</h1>
          <Badge variant="outline">Proxy Active</Badge>
        </div>
      </header>

      <main className="container mx-auto px-4 py-8">
        <Card className="mb-8">
          <CardHeader>
            <CardTitle>New Thread</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Input
              placeholder="Thread title..."
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleCreate()}
            />
            <Button onClick={handleCreate}>Create</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Threads</CardTitle>
          </CardHeader>
          <CardContent>
            {threads.length === 0 ? (
              <p className="text-muted-foreground">No threads yet.</p>
            ) : (
              <div className="space-y-2">
                {threads.map((t) => (
                  <div key={t.id}>
                    <div className="flex items-center justify-between py-2">
                      <span className="font-medium">{t.title || "Untitled"}</span>
                      <Badge>{t.status}</Badge>
                    </div>
                    <Separator />
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
