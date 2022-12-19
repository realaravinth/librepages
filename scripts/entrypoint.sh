#!/bin/bash

USER_ID=${LOCAL_USER_ID}
echo $USER_ID
LIBREPAGES_USER=librepages

echo "Starting with UID : $USER_ID"
export HOME=/home/$LIBREPAGES_USER
#adduser --disabled-password --shell /bin/bash --home $HOME --uid $USER_ID user
#--uid

if id "$1" &>/dev/null; then
	echo "User $LIBREPAGES_USER exists"
else
	useradd --uid $USER_ID -b /home -m -s /bin/bash $LIBREPAGES_USER
fi

su $LIBREPAGES_USER  -c 'librepages'
