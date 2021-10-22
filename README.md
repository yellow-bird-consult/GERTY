# GERTY
Basic cache using Tokio for storing suspended patients that don't exist yet.

## Inserting a patient 
Inserting a patient is fairy easy. It does not matter what endpoint URL you're using or what HTTP method you're using. Your 
body has to have the following contents:
```
{
    "method": "SET",
    "disease": "some disease",
    "patient": {
        data here . . .
    },
    "chatbot data": b"some binary data"
}
```

## Getting a patient
getting a patient from the database requires the following body: 
```
{
    "method": "GET".
    "disease": "some disease"
}
```
