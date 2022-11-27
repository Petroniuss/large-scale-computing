console.log('Loading function');

const aws = require('aws-sdk');

const s3 = new aws.S3({ apiVersion: '2006-03-01' });

exports.lambdaHandler = async (event, context) => {
    console.log('Received event:', JSON.stringify(event, null, 4));
    const bucket = event.Records[0].s3.bucket.name;
    const key = decodeURIComponent(event.Records[0].s3.object.key.replace(/\+/g, ' '));
    const params = {
        Bucket: bucket,
        Key: key,
    };
    try {
        const { Body } = await s3.getObject(params).promise();
        let objectData = Body.toString('utf-8');

        const now = new Date().toString();
        let newObjectData = `${objectData}\n${now}`;

        let destinationBucket = "lsc-test-bucket-1111231231-destination";
        console.log(`Uploading ${bucket}${key} to ${destinationBucket}${key}.`)
        await s3.putObject(
            {
                Bucket: destinationBucket,
                Body: newObjectData,
                Key: key
            }
        );

        return 'OK';
    } catch (err) {
        console.log(err);
        throw new Error(err);
    }
};