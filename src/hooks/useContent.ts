import { useEffect, useRef, useState } from 'react';
import commands from '@/commands';
import writeHostsToSystem from '@/utils/writeHostsToSystem';

export default function useContent(id?: string) {
  const [[contentId, content], setContent] = useState<
    [string | null, string | null]
  >([null, null]);
  const contentMapRef = useRef<Map<string, string>>(new Map());

  const getSystemHosts = async () => {
    const next: string = await commands.getSystemHosts();
    contentMapRef.current.set('0', next);
    if (id === '0') setContent([id, next]);
  };

  const getHostsContent = async (id?: string) => {
    if (!id) return;
    if (contentMapRef.current.has(id)) {
      const next = contentMapRef.current.get(id)!;
      console.log(1, [id, next]);
      setContent([id, next]);
      return;
    }
    if (id === '0') {
      const next: string = await commands.getSystemHosts();
      contentMapRef.current.set(id, next);
      console.log(2, [id, next]);
      if (id === id) setContent([id, next]);
    } else {
      const next: string = await commands.getHostsContent(id);
      console.log(3, [id, next]);
      contentMapRef.current.set(id, next);
      if (id === id) setContent([id, next]);
    }
  };

  useEffect(() => {
    getHostsContent(id);
  }, [id]);

  return {
    content,
    contentId,
    updateContent: async (id: string, content: string) => {
      await commands.setHostsContent(id, content);
      contentMapRef.current.set(id, content);
      await writeHostsToSystem();
      await getSystemHosts();
    },
    refreshSystemHosts: getSystemHosts
  };
}

export type UseContentReturn = ReturnType<typeof useContent>;
