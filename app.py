from flask import Flask, request, send_from_directory, abort
import requests
from PIL import Image
import os
import re
import magic

app = Flask(__name__)

CDN_ROOT = "cdn_root"
ORIGINAL_SERVER = "https://marshallku.com"


@app.route('/files/<path:filename>')
def serve_static_file(filename):
    # Validate file extension
    if not filename.endswith(('css', 'js', 'ico', 'json', 'woff2', 'woff', 'svg', 'ico')):
        abort(404)

    file_path = os.path.join(CDN_ROOT, 'files', filename)
    if not os.path.exists(file_path):
        # Fetch file from original server
        response = requests.get(f"{ORIGINAL_SERVER}/{filename}")
        if response.status_code == 200:
            os.makedirs(os.path.dirname(file_path), exist_ok=True)
            with open(file_path, 'wb') as f:
                f.write(response.content)
        else:
            abort(404)

    return send_from_directory(os.path.dirname(file_path), os.path.basename(file_path))


@app.route('/images/<path:filename>')
def serve_image(filename):
    if os.path.exists(os.path.join(CDN_ROOT, 'images', filename)):
        print("CACHE HIT!")
        return send_from_directory(os.path.join(CDN_ROOT, 'images'), filename)

    original_filename = filename
    resize_width = re.search(r'=w(\d+)$', filename)
    width = 0

    if resize_width:
        original_filename = original_filename.replace(resize_width.group(), '')
        width = resize_width.group(1)

    convert_to_webp = original_filename.endswith('.webp')

    if convert_to_webp:
        original_filename = original_filename.replace('.webp', '')

    file_path = os.path.join(CDN_ROOT, 'images', filename)
    original_file_path = os.path.join(CDN_ROOT, 'images', original_filename)
    if not os.path.exists(original_file_path):
        # Fetch image from original server
        response = requests.get(f"{ORIGINAL_SERVER}/{original_filename}")
        if response.status_code == 200:
            os.makedirs(os.path.dirname(original_file_path), exist_ok=True)
            with open(original_file_path, 'wb') as f:
                f.write(response.content)
        else:
            abort(404)

    if width or convert_to_webp:
        img = Image.open(original_file_path)
        if width:
            mime_magic = magic.Magic(mime=True)
            mimetype = mime_magic.from_file(original_file_path)

            w_percent = (int(width)/float(img.size[0]))
            h_size = int((float(img.size[1])*float(w_percent)))
            img = img.resize((int(width), h_size), Image.ANTIALIAS)
            img.save(file_path, mimetype.split('/')[1], quality=100)
            return send_from_directory(os.path.dirname(file_path), os.path.basename(file_path), mimetype=mimetype)
        if convert_to_webp:
            img.save(file_path, 'WEBP', quality=100)
            return send_from_directory(os.path.dirname(file_path), os.path.basename(file_path), mimetype='image/webp')

    return send_from_directory(os.path.dirname(original_file_path), os.path.basename(original_file_path))


if __name__ == '__main__':
    app.run(port=41890)
