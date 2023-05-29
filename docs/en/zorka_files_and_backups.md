# The Algorithm

This project has a unique non-standarized backup mechanism that is unique to this project.

Each time the process starts, Zorka generates a unique identifier that is unique to the specific instance of the process.

This identifier is stored in a shared file accessible to all instances, that survives process restarts in a persistent storage location. This might be a virtual volume for a virtual process or simply a path on a bare metal node.

When creating a backup file, the previously loaded files are deprecated. This ensures uniqueness across different instances and allows A to override previuos data entries and B to deprecate old deleted entries.

When loading backups, identifiers are retrieved from the persistent storage and used to filter and identify backups that are relevant.

# The .zorka file format

Format: `<UUID> [[ACTION: <UUID>,<UUID>,...], ...]`. Each entry is delimited by the newline `\n` separator.

A example would be:
```zorka
e7d6e4c3-a145-4f5b-bc8c-91dc2aef3e8d
58e49a72-6b65-4ff4-832c-56e29ab840d3 DEPRECATED: e7d6e4c3-a145-4f5b-bc8c-91dc2aef3e8d
ddbb0003-e1f5-4c5e-a79c-26a5c28a0a6f DEPRECATED: e7d6e4c3-a145-4f5b-bc8c-91dc2aef3e8d
6f836a27-8ebd-4abf-927a-1ab78b571b4c DEPRECATED: e7d6e4c3-a145-4f5b-bc8c-91dc2aef3e8d,ddbb0003-e1f5-4c5e-a79c-26a5c28a0a6f
5b909ea9-407a-429b-9e81-fe530d615e74 DEPRECATED: 6f836a27-8ebd-4abf-927a-1ab78b571b4c

```

Explanation:
- `<UUID>` represents the UUID for the current entry.
- `[DEPRECATED: <UUID>,<UUID>,...]` is an optional section indicating which UUIDs that this entry deprecates.

Notice that this structure is inherently recursive. Each entry can deprecate previous entries, which also can deprecate entries.
A entry deprecates other entries if the line is longer than 36 characters. Multiple entries can be deprecated with comma seperation and start from the 49th position.
