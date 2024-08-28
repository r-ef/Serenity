import requests

if __name__ == "__main__":
    for _ in range(100):
        url = "http://127.0.0.1:8000/mine"
        response = requests.post(url, json={"address": "blah"})
        print(response.text)