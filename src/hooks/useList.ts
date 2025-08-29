import { useEffect, useState } from 'react';
import commands from '@/commands';
import { Item } from '@/typing';
import useEvent from '@/hooks/useEvent';
import { EVENTS } from '@/events';

const systemHostItem: Item = {
  id: '0',
  name: 'system',
  on: true,
  system: true,
};

export default function useList() {
  const [list, setList] = useState<Item[]>([]);
  const [current, setCurrent] = useState<Item | null>(systemHostItem);

  useEvent<Item[]>(EVENTS.LIST_UPDATE, (event) => {
    const list = event.payload;
    setList(list);
  });

  useEffect(() => {
    const init = async () => {
      const list: Item[] = await commands.getList();
      setList(list);
    };
    init();
  }, []);

  const updateList = async (next: Item[]) => {
    const res = await commands.setList(next);
    if (res) setList(next);
  };

  return {
    list: [systemHostItem, ...list],
    current,
    setCurrent,
    updateList: commands.setList,
    createItem: async (item: Item) => {
      const next = [...list, item];
      return updateList(next);
    },
    updateItem: async (id: string, data: Partial<Item>) => {
      const next = list.map((item) =>
        item.id === id ? { ...item, ...data } : item
      );
      return updateList(next);
    },
    deleteItem: async (id: string) => {
      if (current?.id === id) {
        const index = list.findIndex((item) => item.id === id);
        if (index !== -1) {
          setCurrent(list[index - 1] || list[index + 1] || null);
        }
      }
      const next = list.filter((item) => item.id !== id);
      return updateList(next);
    },
  };
}

export type UseListReturn = ReturnType<typeof useList>;
