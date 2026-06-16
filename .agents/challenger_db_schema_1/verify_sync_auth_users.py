#!/usr/bin/env python3
import subprocess
import json
import sys

DB_URL = "postgresql://postgres:postgres@127.0.0.1:54322/postgres"

def run_sql(query):
    cmd = ["psql", DB_URL, "-t", "-A", "-c", query]
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        raise Exception(f"SQL execution failed: {res.stderr}\nQuery: {query}")
    return res.stdout.strip()

def run_sql_json(query):
    clean_query = query.rstrip('; ')
    json_query = f"SELECT coalesce(json_agg(t), '[]'::json) FROM ({clean_query}) t;"
    stdout = run_sql(json_query)
    try:
        return json.loads(stdout)
    except json.JSONDecodeError as e:
        raise Exception(f"Failed to parse JSON output: '{stdout}'. Error: {e}")

def cleanup():
    run_sql("DELETE FROM auth.users WHERE id::text LIKE '00000000-0000-0000-0000-00000000%';")

def test_happy_path():
    print("--- Happy Path Test ---")
    user_id = "00000000-0000-0000-0000-000000000001"
    email = "happy@example.com"
    meta = '{"name": "Happy User", "username": "happyuser"}'
    
    try:
        run_sql(f"INSERT INTO auth.users (id, email, raw_user_meta_data) VALUES ('{user_id}', '{email}', '{meta}'::jsonb);")
        res = run_sql_json(f"SELECT id, username, name, email FROM public.players WHERE id = '{user_id}';")
        assert len(res) == 1, f"Expected 1 player, got {len(res)}"
        player = res[0]
        assert player['id'] == user_id, f"Expected ID {user_id}, got {player['id']}"
        assert player['email'] == email, f"Expected email {email}, got {player['email']}"
        assert player['name'] == "Happy User", f"Expected name 'Happy User', got {player['name']}"
        assert player['username'] == "happyuser", f"Expected username 'happyuser', got {player['username']}"
        print("[PASS] Happy Path test passed.")
        return True
    except Exception as e:
        print(f"[FAIL] Happy Path test failed: {e}")
        return False

def test_fallback_path():
    print("--- Fallback Path Test ---")
    user_id = "00000000-0000-0000-0000-000000000002"
    
    try:
        run_sql(f"INSERT INTO auth.users (id) VALUES ('{user_id}');")
        res = run_sql_json(f"SELECT id, username, name, email FROM public.players WHERE id = '{user_id}';")
        assert len(res) == 1, f"Expected 1 player, got {len(res)}"
        player = res[0]
        assert player['id'] == user_id
        assert player['email'] is None
        assert player['name'] == "Player", f"Expected name 'Player', got {player['name']}"
        assert player['username'] == f"player_00000000", f"Expected username 'player_00000000', got {player['username']}"
        print("[PASS] Fallback Path test passed.")
        return True
    except Exception as e:
        print(f"[FAIL] Fallback Path test failed: {e}")
        return False

def test_trimming_path_spaces():
    print("--- Trimming Path Test (Spaces Only) ---")
    user_id = "00000000-0000-0000-0000-000000000003"
    meta = '{"name": "   ", "username": "   "}'
    
    try:
        run_sql(f"INSERT INTO auth.users (id, raw_user_meta_data) VALUES ('{user_id}', '{meta}'::jsonb);")
        res = run_sql_json(f"SELECT id, username, name, email FROM public.players WHERE id = '{user_id}';")
        assert len(res) == 1, f"Expected 1 player, got {len(res)}"
        player = res[0]
        assert player['id'] == user_id
        assert player['email'] is None
        assert player['name'] == "Player", f"Expected name 'Player' after trimming spaces, got '{player['name']}'"
        assert player['username'] == f"player_00000000", f"Expected username 'player_00000000' after trimming spaces, got '{player['username']}'"
        print("[PASS] Trimming Path (Spaces Only) test passed.")
        return True
    except Exception as e:
        print(f"[FAIL] Trimming Path (Spaces Only) test failed: {e}")
        return False

def test_trimming_path_whitespace():
    print("--- Trimming Path Test (Tabs and Newlines) ---")
    user_id = "00000000-0000-0000-0000-000000000004"
    meta = '{"name": " \\t\\n ", "username": " \\t\\n "}'
    
    try:
        run_sql(f"INSERT INTO auth.users (id, raw_user_meta_data) VALUES ('{user_id}', '{meta}'::jsonb);")
        res = run_sql_json(f"SELECT id, username, name, email FROM public.players WHERE id = '{user_id}';")
        assert len(res) == 1, f"Expected 1 player, got {len(res)}"
        player = res[0]
        assert player['id'] == user_id
        assert player['email'] is None
        assert player['name'] == "Player", f"Expected name 'Player' after trimming tabs/newlines, got '{player['name']}'"
        assert player['username'] == f"player_00000000", f"Expected username 'player_00000000' after trimming tabs/newlines, got '{player['username']}'"
        print("[PASS] Trimming Path (Tabs and Newlines) test passed.")
        return True
    except Exception as e:
        print(f"[FAIL] Trimming Path (Tabs and Newlines) test failed: {e}")
        return False

def test_conflict_path():
    print("--- Conflict Path Test ---")
    base_user_id = "00000000-0000-0000-0000-000000000100"
    meta = '{"username": "conflict_user", "name": "Conflict"}'
    
    try:
        # Basic suffixing
        run_sql(f"INSERT INTO auth.users (id, email, raw_user_meta_data) VALUES ('{base_user_id}', 'c1@example.com', '{meta}'::jsonb);")
        res = run_sql_json(f"SELECT username FROM public.players WHERE id = '{base_user_id}';")
        assert res[0]['username'] == 'conflict_user', f"Expected 'conflict_user', got {res[0]['username']}"
        
        id2 = "00000000-0000-0000-0000-000000000101"
        run_sql(f"INSERT INTO auth.users (id, email, raw_user_meta_data) VALUES ('{id2}', 'c2@example.com', '{meta}'::jsonb);")
        res2 = run_sql_json(f"SELECT username FROM public.players WHERE id = '{id2}';")
        assert res2[0]['username'] == 'conflict_user_1', f"Expected 'conflict_user_1', got {res2[0]['username']}"
        
        id3 = "00000000-0000-0000-0000-000000000102"
        run_sql(f"INSERT INTO auth.users (id, email, raw_user_meta_data) VALUES ('{id3}', 'c3@example.com', '{meta}'::jsonb);")
        res3 = run_sql_json(f"SELECT username FROM public.players WHERE id = '{id3}';")
        assert res3[0]['username'] == 'conflict_user_2', f"Expected 'conflict_user_2', got {res3[0]['username']}"
        print("[PASS] Conflict Path (basic suffixing) passed.")
        
        # 102 conflicting insertions stress test
        print("Stress-testing: inserting up to 102 users with same base username...")
        for i in range(3, 102):
            uid = f"00000000-0000-0000-0000-000000000{200 + i:03d}"
            run_sql(f"INSERT INTO auth.users (id, email, raw_user_meta_data) VALUES ('{uid}', 'c{i+1}@example.com', '{meta}'::jsonb);")
        
        last_uid = "00000000-0000-0000-0000-000000000301" # 200 + 101
        res_last = run_sql_json(f"SELECT username FROM public.players WHERE id = '{last_uid}';")
        last_username = res_last[0]['username']
        print(f"Generated 102nd username: {last_username}")
        assert last_username.startswith("conflict_user_"), f"Expected to start with 'conflict_user_', got '{last_username}'"
        assert len(last_username) == 20, f"Expected length 20, got {len(last_username)}"
        hex_part = last_username.split("_")[-1]
        assert len(hex_part) == 6, f"Expected 6 char hex suffix, got '{hex_part}'"
        int(hex_part, 16) # Verify it is hex
        print("[PASS] Conflict Path (stress-test random hash fallback) passed.")
        return True
    except Exception as e:
        print(f"[FAIL] Conflict Path test failed: {e}")
        return False

def test_integrity_path():
    print("--- Integrity Path Test ---")
    user_id = "00000000-0000-0000-0000-000000000005"
    
    try:
        run_sql(f"INSERT INTO auth.users (id, email) VALUES ('{user_id}', 'delete@example.com');")
        res = run_sql_json(f"SELECT count(*) FROM public.players WHERE id = '{user_id}';")
        assert res[0]['count'] == 1, "Expected player to be created"
        
        run_sql(f"DELETE FROM auth.users WHERE id = '{user_id}';")
        res2 = run_sql_json(f"SELECT count(*) FROM public.players WHERE id = '{user_id}';")
        assert res2[0]['count'] == 0, "Expected player to be deleted via cascade"
        print("[PASS] Integrity Path test passed.")
        return True
    except Exception as e:
        print(f"[FAIL] Integrity Path test failed: {e}")
        return False

def verify_security():
    print("--- Security Configuration Verification ---")
    try:
        res = run_sql_json("SELECT prosecdef, proconfig FROM pg_proc WHERE proname = 'handle_new_user';")
        assert len(res) == 1, "Expected function handle_new_user to exist"
        func = res[0]
        assert func['prosecdef'] is True, f"Expected SECURITY DEFINER (prosecdef=True), got {func['prosecdef']}"
        config = func['proconfig']
        assert config is not None, "Expected proconfig to not be None"
        search_path_found = False
        for conf in config:
            if conf.startswith("search_path="):
                search_path_found = True
                val = conf.split("=", 1)[1]
                assert val == "pg_catalog, public", f"Expected search_path to be 'pg_catalog, public', got '{val}'"
        assert search_path_found, "Expected search_path configuration to be present in proconfig"
        print("[PASS] Security Definer and Search Path verification passed.")
        return True
    except Exception as e:
        print(f"[FAIL] Security Configuration verification failed: {e}")
        return False

def main():
    print("Starting Trigger Verification...")
    results = {}
    try:
        cleanup()
        results['Happy Path'] = test_happy_path()
        cleanup()
        results['Fallback Path'] = test_fallback_path()
        cleanup()
        results['Trimming Path (Spaces)'] = test_trimming_path_spaces()
        cleanup()
        results['Trimming Path (Tabs & Newlines)'] = test_trimming_path_whitespace()
        cleanup()
        results['Conflict Path'] = test_conflict_path()
        cleanup()
        results['Integrity Path'] = test_integrity_path()
        cleanup()
        results['Security Check'] = verify_security()
    finally:
        cleanup()
        
    print("\n=== Test Results Summary ===")
    all_passed = True
    for test_name, passed in results.items():
        status = "PASS" if passed else "FAIL"
        print(f"{test_name}: {status}")
        if not passed:
            all_passed = False
            
    if all_passed:
        print("\nAll tests passed!")
        sys.exit(0)
    else:
        print("\nSome tests failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()
