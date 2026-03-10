from ..agent_loop import run_agent
from ..memory_store.short_term import ShortTermMemory


def run_cli():
    stm = ShortTermMemory()
    print("Mini-claw CLI. Ctrl+C to exit.")
    while True:
        try:
            user = input("> ")
        except (EOFError, KeyboardInterrupt):
            print("\nbye")
            break
        if not user.strip():
            continue
        reply = run_agent(user, stm)
        print(reply)
