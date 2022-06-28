import fs from 'node:fs';
import path from 'node:path';
import { createDeployment, createFunction, getDeploymentConfig, writeDeploymentConfig } from '../utils/deployments';
import { logDebug, logError, logSuccess } from '../utils/logger';
import inquirer from 'inquirer';
import { SUPPORTED_EXTENSIONS } from '../utils/constants';
import { authToken } from '../auth';
import { trpc } from '../trpc';

export async function deploy(file: string) {
  const fileToDeploy = path.join(process.cwd(), file);

  if (!fs.existsSync(fileToDeploy) || fs.statSync(fileToDeploy).isDirectory()) {
    logError(`File ${fileToDeploy} does not exist.`);
    return;
  }

  const extension = path.extname(fileToDeploy);

  if (!SUPPORTED_EXTENSIONS.includes(extension)) {
    logError(`Extension ${extension} is not supported: ${SUPPORTED_EXTENSIONS.join(', ')}`);
    return;
  }

  const config = getDeploymentConfig(fileToDeploy);

  if (!config) {
    logDebug('No deployment config found...');

    const { link } = await inquirer.prompt({
      type: 'confirm',
      name: 'link',
      message: 'Link to an existing function?',
    });

    const organizations = await trpc(authToken).query('organizations.list');

    const { organization } = await inquirer.prompt([
      {
        type: 'list',
        name: 'organization',
        message: 'Select the organization to deploy to',
        choices: organizations.map(({ name, id }) => ({
          name,
          value: id,
        })),
      },
    ]);

    if (link) {
      const functions = await trpc(authToken).query('functions.list');

      const { func } = await inquirer.prompt([
        {
          type: 'list',
          name: 'func',
          message: 'Select the function to deploy',
          choices: functions.map(({ name, id }) => ({
            name,
            value: id,
          })),
        },
      ]);

      createDeployment(func, fileToDeploy);
      writeDeploymentConfig(fileToDeploy, { functionId: func, organizationId: organization });
      logSuccess(`Function deployed.`);
    } else {
      const { name } = await inquirer.prompt([
        {
          type: 'input',
          name: 'name',
          message: 'What is the name of the function?',
        },
      ]);

      logDebug(`Creating function ${name}...`);
      const func = await createFunction(name, fileToDeploy);
      writeDeploymentConfig(fileToDeploy, { functionId: func.id, organizationId: organization });
      logSuccess(`Function ${name} created.`);
    }

    return;
  }

  createDeployment(config.functionId, fileToDeploy);
  logSuccess(`Function deployed.`);
}
