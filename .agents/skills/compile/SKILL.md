---
name: compile
description: Triggers RustPress compilation and server restart on the macOS host from within the Docker container. Invoke this skill when the user requests "/compile" or asks to recompile, rebuild, or restart the dev server.
---

# Compile Skill

This skill allows the agent to signal the macOS host to trigger cargo compilation and restart the development server.

## Execution Steps

1. Run the terminal command to touch `.rebuild-trigger` in the project root:
   ```bash
   touch /workspace/rustpress/.rebuild-trigger
   ```
2. Notify the user that the compilation and restart signal has been sent to the host development server.
