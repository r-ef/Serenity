import os
import requests

if __name__ == "__main__":
    base_url = os.environ.get("SERENITY_BASE_URL", "http://127.0.0.1:8000")
    for _ in range(100):
        url = f"{base_url}/mine"
        response = requests.post(url, json={"address": "blah"})
        print(response.text)