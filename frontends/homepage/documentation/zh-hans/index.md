# YYDB 文档

YYDB 是一款以 **VOS** 为原生数据语言的单文件本地数据库。它把结构化记录、键值数据、大文件和向量载荷放在同一个数据边界里，适合桌面应用、本地优先工具、游戏编辑器与边缘服务。

## 一个文件，完整的数据能力

- **单文件数据库**：应用数据集中在一个 `.yydb` 文件中，便于创建、移动和备份。
- **可靠事务**：WAL 与崩溃恢复保护写入过程，重新打开即可继续工作。
- **VOS schema**：类型、引用、主键和唯一约束直接写进数据模型。
- **原生文件支持**：大文件使用分片对象存储，支持按范围读取，无需另接文件服务。
- **向量载荷**：向量与业务记录共享一致的数据生命周期和对象存储。
- **本地低延迟**：Rust 应用可进程内嵌入，TypeScript 应用只需一个 npm 依赖。

## VOS 数据模型

VOS 让数据关系在 schema 中保持清晰：

```vos
table User {
  @@user_id: uuid
  @user_name: utf8
  avatar: file?
  embedding: vector<f32, 768>?
}

table Post {
  @@post_id: uuid
  author: &User
  title: utf8
}
```

`@@user_id` 是主键，`@user_name` 保证唯一，`&User` 表示对 `User` 记录的引用。文件与向量也是模型中的原生字段，不需要把数据拆到另一套 API。

## 选择接入方式

TypeScript / Node.js 应用从 [`@yydb/yydb`](/d/guide/embed-and-serve) 开始；Rust 宿主可以直接嵌入 YYDB。浏览器工具或需要自行管理连接的客户端，可使用底层线协议客户端。
