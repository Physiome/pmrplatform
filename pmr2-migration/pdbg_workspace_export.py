import json

def timestamp(dt):
    if dt:
        return int(dt.timeTime())

results = []
records = app.pmr.portal_catalog(portal_type='Workspace', review_state='published')
for record in records:
    path = record.getPath()
    obj = record.getObject()
    result = {
        # cannot use id naively as alias due to `/w/{user}/`
        # 'alias': record.id
        # instead, drop site prefix and sub out strings
        'alias': path
            .replace('/pmr/', '')
            # this should maintain the id for default case
            .replace('workspace/', '')
            # drop the prefix for users and replace sep with `-`
            .replace('w/', '')
            .replace('/', '-'),
        'description': record.Title,
        'long_description': record.Description,
        'path': path,
        'url': path.replace('/pmr/', 'https://models.physiomeproject.org/') + '/',
        'workflow_state': record.review_state,

        "creation_date": timestamp(obj.creation_date),
        "effective_date": timestamp(obj.effective_date),
    }
    results.append(result)

with open('/tmp/workspace_dump.json', 'w') as fd:
    json.dump(results, fd)
