# Page snapshot

```yaml
- generic [active] [ref=e1]:
  - generic [ref=e2]:
    - img [ref=e3]
    - heading "Dioxus" [level=2] [ref=e7]
    - heading "web | plon" [level=2] [ref=e8]
  - generic [ref=e9]:
    - heading "Oops! The build failed." [level=1] [ref=e10]
    - paragraph [ref=e11]:
      - generic [ref=e12]: compiling plon
      - text: "encountered an error:"
    - generic [ref=e13]: "Cargo build failed, signaled by the compiler. Toggle tracing mode (press `t`) for more information."
  - heading "docs | guides | help" [level=4] [ref=e15]:
    - link "docs" [ref=e16] [cursor=pointer]:
      - /url: https://docs.rs/dioxus/latest/dioxus/
    - text: "|"
    - link "guides" [ref=e17] [cursor=pointer]:
      - /url: https://dioxuslabs.com/learn
    - text: "|"
    - link "help" [ref=e18] [cursor=pointer]:
      - /url: https://discord.gg/KrGKhBbU6s
```