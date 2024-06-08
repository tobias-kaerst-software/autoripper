import { useQuery } from '@tanstack/react-query';
import { FC, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';

import { FormControl, FormField, FormItem, FormLabel } from '$/components/common/ui/form';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '$/components/common/ui/select';
import { cn } from '$/lib/utils';
import { useMediaStore } from '$/pages/Homepage/stores/useMediaStore';
import { qualityProfileQuery } from '$/services/management';

import type { MetadataFormControl } from '$/pages/Homepage/components/SettingsDialog/components/SettingsForm';

interface Props {
  form: MetadataFormControl;
}

export const QualityProfileSelection: FC<Props> = ({ form }) => {
  const { t } = useTranslation();
  const { data, isLoading, isRefetching } = useQuery(qualityProfileQuery({ media_type: form.getValues('type') }));

  const rippingInProgress = useMediaStore(useShallow((state) => state.rippingInProgress));

  const loading = isLoading || isRefetching;

  useEffect(() => {
    if ((data?.length ?? 0) > 0 && !form.getValues('qualityProfile')) form.setValue('qualityProfile', data![0].id);
  }, [data, form]);

  return (
    <FormField
      control={form.control}
      name='qualityProfile'
      render={({ field, fieldState }) => (
        <FormItem className='w-full'>
          <FormLabel className='flex items-center gap-1 text-sm'>
            <span>{t('homepage.qualityProfile.title')}</span>
          </FormLabel>
          <Select disabled={loading || rippingInProgress} onValueChange={field.onChange} value={field.value}>
            <FormControl>
              <div className='flex gap-3'>
                <SelectTrigger
                  className={cn('w-full', fieldState.error && 'border-red-500')}
                  isLoading={loading}
                  ref={field.ref}
                >
                  <SelectValue placeholder={t('homepage.qualityProfile.placeholder')} />
                </SelectTrigger>
              </div>
            </FormControl>
            <SelectContent>
              {data?.map((profile) => (
                <SelectItem key={profile.id} value={profile.id} className='cursor-pointer'>
                  <span>{profile.name}</span>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </FormItem>
      )}
    />
  );
};