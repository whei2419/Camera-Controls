# PHP Upload Error 3 (UPLOAD_ERR_PARTIAL) - Troubleshooting

If you're still getting error 3 from Laravel, check these PHP settings:

## 1. Check php.ini Settings

Find your `php.ini` file and verify these values:

```ini
upload_max_filesize = 10M      # Increase to at least 10M
post_max_size = 12M            # Must be larger than upload_max_filesize
max_input_time = 300           # Max time for parsing input (seconds)
memory_limit = 256M            # Sufficient memory
```

## 2. Restart PHP-FPM / Apache

After changing php.ini, restart your web server:

```bash
# For Apache
sudo systemctl restart apache2

# Or if using PHP-FPM
sudo systemctl restart php8.1-fpm  # Adjust version number
```

## 3. Check Laravel Log for Exact File Size

Your logs show:
- Content-Length: 129545 bytes (~126 KB)
- File size in $_FILES: 0 bytes (means partial upload)

## 4. Quick PHP Info Test

Create a test file to check current settings:

```php
<?php
phpinfo();
```

Look for:
- `upload_max_filesize`
- `post_max_size`
- `upload_tmp_dir` (should exist and be writable)

## 5. Alternative: Increase Limits in .htaccess

If you can't edit php.ini, add to `.htaccess`:

```apache
php_value upload_max_filesize 10M
php_value post_max_size 12M
php_value max_input_time 300
```
