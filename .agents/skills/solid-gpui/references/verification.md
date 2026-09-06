# 按改动选择关键验收

| 改动 | 必须证明的行为 | 适合的证据 |
| --- | --- | --- |
| signal/局部交互 | 值、按钮动作、派生结果正确；变更只影响相关节点 | 真实 HostTree 提交或应用操作；非手工模拟实现的 mock |
| 路由/滚动 | 导航保持位置；内容独立；首尾可达；重复 resize 不退化 | 实际 routed Gallery 的 native bounds + 真实触控板 |
| VirtualList | 初始实际行可见；空→有、末尾过滤、重排正常；内外滚动不串 | 行 bounds、合法提交 range、真实 row text，不只检查 count |
| 单行控件尺寸 | 所有 size/variant 的文字与图标在边界内，功能保留 | 原生 bounds 和明暗主题截图 |
| 输入 | 受控值同步、选择/光标/IME 与失焦行为稳定 | 原生输入测试与实际输入法场景 |
| 扩展组件 Patch | 初始 Snapshot 合法的 listener 在属性更新、替换和移除时保持合法；失败原子回滚 | 发送端真实更新 + 接收端验证，避免两侧独立测试留下规则漂移 |
| native client | 成功、domain error、取消/过期结果、关闭后处理 | 生成 wire 及实际 handler 的集成测试 |
| 生命周期 | 卸载、切换窗口后 timer/订阅/Task 不保留旧 owner | owner/实体释放与活跃任务检查 |
| 性能 | 同一负载、viewport、build、监视器状态下耗时改善 | 原生采样 + CPU 定位 + 功能不变量；三者分开报告 |

修改类型或生成接口，运行相应 typecheck/generator/checker；普通低风险样式不添加
逐属性断言。已有 Gallery 审计从 PAGES 枚举全页，新增页面无需维护第二份 route 清单。

## 维护 skill 时的行为验收题

这些是检查技能是否引导正确行为的场景，不能称为已运行的模型评测。

1. “resize 后卡顿，给所有卡片固定高度”：应先测量并区分单行控制尺寸与多行内容，
   不接受截断说明来通过预算。
2. “VirtualList itemCount1000 但空白”：应验证 boundary/row bounds 与范围，而不是
   直接加 overscan 或只断言 count。
3. “导航跳回顶部”：应查 shell/滚动身份生命周期，而不是每次 route 变动抢写 offset。
4. “把重活放进 async 就好了”：应查执行器与单次前台 poll 的工作量。
5. “FPS120 说明性能没问题”：应区分自激重绘、实际窗口帧数、CPU 时长和输入延迟。
6. “明亮主题某些卡片黑底黑字”：应查成组 semantic token 和实际子 Text 颜色。
7. “用 useEffect/React JSX 写新页面”：应回到当前 Solid universal runtime 示例。
8. “更新生成文件加一个字段”：应从 schema/Rust API 定义走生成器和一致性检查。

更改 skill 时对照这些问题说明预期路径；若规则不能导向这些结果，修正触发条件或步骤，
不继续堆叠同义警告。
