迁移依赖合并草案

目标：把 `SwitchHosts` 渲染端所需的前端依赖合并到根 `package.json`，以便在外层 Tauri 项目中运行 renderer 代码。

注意项
- 根项目当前使用 React 19，`SwitchHosts` 原项目使用 React 18.x；多数库兼容，但在引入像 `@chakra-ui/react`、`react-router-dom` 等需要核对支持的 React 版本。请在合并前在分支上测试。
- 根项目已包含 `@tauri-apps/api`，无需合并 Electron 特有依赖（如 `electron`、`electron-updater` 等）。这些需要改写为 Tauri API。

建议合并的运行时依赖（从 `SwitchHosts/package.json` 摘取并保留版本）：

必需/优先（前端运行时）
- axios@1.8.2
- dayjs@1.11.12
- lodash@4.17.21
- uuid@10.0.0
- react-router-dom@7.5.2
- react-icons@5.2.1
- jotai@2.9.1
- ahooks@3.8.0
- clsx@2.1.1
- react-window@1.8.10
- codemirror@5.65.17

UI / 样式相关（按需求选择）
- @chakra-ui/react@2.8.2
- @emotion/react@11.13.0
- @emotion/styled@11.13.0
- sass@1.77.8

构建/开发时依赖（按需）
- vite-plugin-svgr@4.3.0
- vite-plugin-static-copy@2.3.1
- vite-tsconfig-paths@4.3.2

TypeScript 类型（devDependencies）
- @types/react-window@1.8.8
- @types/uuid@10.0.0
- @types/codemirror@5.60.15

安装建议（项目使用 pnpm；若你使用 npm/yarn，请相应替换命令）

示例：先安装运行时依赖

```bash
pnpm add axios@1.8.2 dayjs@1.11.12 lodash@4.17.21 uuid@10.0.0 react-router-dom@7.5.2 react-icons@5.2.1 jotai@2.9.1 ahooks@3.8.0 clsx@2.1.1 react-window@1.8.10 codemirror@5.65.17
```

示例：安装 UI 与 dev 依赖

```bash
pnpm add -D @chakra-ui/react@2.8.2 @emotion/react@11.13.0 @emotion/styled@11.13.0 sass@1.77.8 vite-plugin-svgr@4.3.0 vite-plugin-static-copy@2.3.1 vite-tsconfig-paths@4.3.2 @types/react-window@1.8.8 @types/uuid@10.0.0 @types/codemirror@5.60.15
```

后续步骤建议（迁移顺序）
1. 安装依赖（在新分支上）并启动 dev，观察首批编译错误。
2. 识别并替换 Electron 特有 API（ipc、updater、window-state）为 Tauri 对应实现，并在 `src-tauri` 中添加必要的命令/实现占位。
3. 调整路由或入口，将 `src/switchhosts/renderer/index.tsx`（或迁移后的入口）挂载到外层应用的路由（例如 /switchhosts）。
4. 逐项修复样式、静态资源路径问题，测试功能性（hosts 管理、导入导出、开关等）。

如果你同意，我可以接着把 `SwitchHosts/src/renderer` 中的入口文件（`index.tsx`/`index.html`）接入外层 `src/main.tsx` 路由并做首轮编译修复。
