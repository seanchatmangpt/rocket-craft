with open("/Users/sac/rocket-craft/ggen-validation-tests/core.ttl", "rb") as f:
    content = f.read()
    
start = content.find(b"gundam:GundamWorld")
if start != -1:
    chunk = content[start:start+500]
    print("Hex dump:")
    print(chunk.hex())
    print("Text:")
    print(chunk.decode("utf-8", errors="replace"))
else:
    print("Not found")
