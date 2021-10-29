#!/bin/bash

USER_ID=${LOCAL_USER_ID}
echo $USER_ID

echo "Starting with UID : $USER_ID"
export HOME=/home/user
#adduser --disabled-password --shell /bin/bash --home $HOME --uid $USER_ID user
#--uid
sudo useradd --uid $USER_ID -b /home -m -s /bin/bash user
su - user 
pages
