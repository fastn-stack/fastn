from pyquery import PyQuery as pq
import requests
import sys

def test_ssr(url):
    user_agents = [
        "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123 Safari/537.36"
    ]

    for user_agent in user_agents:
        headers = {'User-Agent': user_agent}
        response = requests.get(url, headers=headers)
        doc = pq(response.text)

        body_tag = doc('body')

        if "Googlebot" in user_agent:
            assert len(body_tag.children()) > 0
        else:
            assert not body_tag.children()

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python test-ssr.py <url>")
        sys.exit(1)
    
    url = sys.argv[1]
    test_ssr(url.strip())
