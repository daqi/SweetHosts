import { useStore } from '@/store';
import cx from 'clsx';
import styles from './index.module.less';
import { Item } from '@/typing';
import { v4 as uuidV4 } from 'uuid';
import { useRef, useState } from 'react';
import { confirm } from '@tauri-apps/plugin-dialog';
import writeHostsToSystem from '@/utils/writeHostsToSystem';

function ListItem(props: {
  item: Item;
  active: boolean;
  onClick: () => void;
  onCheck: (checked: boolean) => void;
  onEdit: (next: string) => void;
  onDelete: () => void;
}) {
  const { item, active, onClick, onCheck, onEdit, onDelete } = props;
  const [editing, setEditing] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);

  const handleEdit = () => {
    if (item.system) return;
    if (!active) return;
    setEditing(true);
    setTimeout(() => {
      const input: HTMLInputElement | null | undefined =
        wrapperRef.current?.querySelector(`.${styles.input}`);
      if (input) {
        input.focus();
        input.select();
      }
    }, 0);
  };

  const handleInputKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    const value = e.currentTarget.value.trim();
    if (e.key === 'Enter') {
      onEdit(value);
      setEditing(false);
    }
    if (e.key === 'Escape') {
      setEditing(false);
    }
  };

  return (
    <div
      key={item.id}
      onClick={onClick}
      className={cx(styles.listItem, {
        [styles.active]: active,
      })}
      ref={wrapperRef}
    >
      <span className={styles.nameWrapper}>
        <input
          className={styles.checkbox}
          type="checkbox"
          checked={item.on}
          onChange={(e) => onCheck(e.target.checked)}
          readOnly={item.system}
        />
        {editing ? (
          <input
            defaultValue={item.name}
            className={styles.input}
            onChange={(e) => setEditing((e.target as any).value)}
            onKeyDown={handleInputKeyDown}
            onBlur={() => setEditing(false)}
          />
        ) : (
          <>
            <span
              className={cx(styles.name, { [styles.system]: item.system })}
              onDoubleClick={handleEdit}
            >
              {item.name}
            </span>
            {item.system ? null : (
              <>
                <span className={styles.edit} onClick={handleEdit}>
                  e
                </span>
                <span className={styles.delete} onClick={onDelete}>
                  d
                </span>
              </>
            )}
          </>
        )}
      </span>
    </div>
  );
}

export default function List() {
  const {
    list,
    createItem,
    updateItem,
    deleteItem,
    current,
    setCurrent,
    refreshSystemHosts,
  } = useStore();
  return (
    <div className={styles.listWrapper}>
      <div className={styles.list}>
        {list?.map((el) => (
          <ListItem
            key={el.id}
            item={el}
            active={current?.id === el.id}
            onClick={() => setCurrent(el)}
            onCheck={async (checked) => {
              await updateItem(el.id, { on: checked });
              await writeHostsToSystem();
              await refreshSystemHosts();
            }}
            onEdit={(next) => {
              if (next) {
                updateItem(el.id, { name: next });
              }
            }}
            onDelete={async () => {
              console.log('onDelete');
              if (await confirm('Are you sure you want to delete this item?')) {
                await deleteItem(el.id);
                await writeHostsToSystem();
                await refreshSystemHosts();
              }
            }}
          />
        ))}
      </div>
      <form
        className={styles.add}
        onSubmit={async (e) => {
          e.preventDefault();
          const name = (e.target as any).name.value;
          if (name.trim()) {
            await createItem({ id: uuidV4(), name, on: true });
          }
          (e.target as HTMLFormElement).reset();
        }}
      >
        <input name="name" type="text" className={styles.input} />
        <button type="submit" className={styles.addButton}>
          OK
        </button>
      </form>
    </div>
  );
}
