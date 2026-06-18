import urllib.request
import json

url = 'https://api.github.com/repos/limine-bootloader/limine/releases'
headers = {'User-Agent': 'Mozilla/5.0'}
req = urllib.request.Request(url, headers=headers)
try:
    with urllib.request.urlopen(req) as response:
        releases = json.loads(response.read().decode('utf-8'))
        for rel in releases:
            tag = rel.get('tag_name')
            assets = rel.get('assets', [])
            has_binary = any('binary' in a.get('name', '') for a in assets)
            if has_binary:
                print(f"Tag: {tag} has binary assets:")
                for a in assets:
                    if 'binary' in a.get('name', ''):
                        print("  ", a.get('name'), "->", a.get('browser_download_url'))
except Exception as e:
    print("Error:", e)
